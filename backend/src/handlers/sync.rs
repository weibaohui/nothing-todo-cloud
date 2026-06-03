//! 同步核心处理模块
//! 采用服务器端合并策略：按标题合并，所有设备共享用户级全局快照

use crate::db::schema::DeviceSnapshots;
use crate::db::schema::device_snapshot::Column as SnapshotColumn;
use crate::error::Result;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// YAML 中的 todo 结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TodoItem {
    pub title: String,
    #[serde(default)]
    pub done: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// 解析后的数据格式
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SyncData {
    #[serde(default)]
    pub todos: Vec<TodoItem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

/// 同步状态响应
#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub user_id: i64,
    pub last_sync_at: String,
}

/// 推送数据请求
#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub device_id: i64,
    /// 数据类型：todos / tags / skills
    pub data_type: String,
    /// YAML 格式的数据
    pub data: String,
}

/// 推送数据响应
#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub success: bool,
    /// 合并后的数据
    pub merged_data: String,
}

/// 拉取数据请求
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub device_id: i64,
    pub data_type: Option<String>,
}

/// 拉取数据响应
#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub device_id: i64,
    pub data_type: String,
    pub data: String,
    pub updated_at: String,
}

/// 获取同步状态
pub async fn status(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullRequest>,
) -> Result<Json<SyncStatus>> {
    let data_type = params.data_type.clone().unwrap_or_else(|| "todos".to_string());

    // 获取设备所属用户的快照
    let device = crate::db::schema::Devices::find_by_id(params.device_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("设备不存在".to_string()))?;

    let latest = find_user_snapshot(&state.db, device.user_id, &data_type).await?;

    let last_sync_at = latest.map(|s| s.created_at.to_rfc3339()).unwrap_or_default();

    Ok(Json(SyncStatus {
        user_id: device.user_id,
        last_sync_at,
    }))
}

/// Push：上传数据并与服务器端合并
pub async fn push(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>> {
    tracing::info!(
        "Push 请求: device_id={}, data_type={}, data_len={}",
        req.device_id,
        req.data_type,
        req.data.len()
    );

    let now = Utc::now();

    // 获取设备所属用户
    let device = crate::db::schema::Devices::find_by_id(req.device_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("设备不存在".to_string()))?;
    let user_id = device.user_id;

    // 解析客户端发送的 YAML 数据
    let client_data: SyncData = serde_yaml::from_str(&req.data)
        .map_err(|e| crate::error::AppError::BadRequest(format!("YAML 解析失败: {}", e)))?;

    // 加载服务器端现有用户数据
    let server_data = if let Some(snapshot) = find_user_snapshot(&state.db, user_id, &req.data_type).await? {
        serde_yaml::from_str(&snapshot.data_payload).unwrap_or_default()
    } else {
        SyncData::default()
    };

    // 按标题合并：服务器端数据优先
    let merged = merge_data(server_data, client_data, &req.data_type);

    // 序列化为 YAML
    let merged_yaml = serde_yaml::to_string(&merged)
        .map_err(|e| crate::error::AppError::Internal(format!("YAML 序列化失败: {}", e)))?;

    // 删除用户旧快照
    delete_user_snapshots(&state.db, user_id, &req.data_type).await?;

    // 存储新快照
    let new_snapshot = crate::db::schema::device_snapshot::ActiveModel {
        device_id: Set(req.device_id),  // 保留最后一个 push 的设备 ID
        version: Set(1),
        data_type: Set(req.data_type.clone()),
        data_payload: Set(merged_yaml.clone()),
        checksum: Set(String::new()),
        created_at: Set(now),
        ..Default::default()
    };
    new_snapshot.insert(&state.db).await?;

    // 记录同步日志
    let sync_log = crate::db::schema::sync_log::ActiveModel {
        device_id: Set(req.device_id),
        action: Set("push".to_string()),
        status: Set("success".to_string()),
        details: Set(Some(format!("data_type={}, merged", req.data_type))),
        created_at: Set(now),
        ..Default::default()
    };
    sync_log.insert(&state.db).await?;

    Ok(Json(PushResponse {
        success: true,
        merged_data: merged_yaml,
    }))
}

/// Pull：拉取服务器端用户全局数据
pub async fn pull(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullRequest>,
) -> Result<Json<PullResponse>> {
    let data_type = params.data_type.clone().unwrap_or_else(|| "todos".to_string());

    tracing::info!("Pull 请求: device_id={}, data_type={}", params.device_id, data_type);

    let now = Utc::now();

    // 获取设备所属用户
    let device = crate::db::schema::Devices::find_by_id(params.device_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("设备不存在".to_string()))?;
    let user_id = device.user_id;

    // 更新设备最后访问时间
    let mut device: crate::db::schema::device::ActiveModel = device.into();
    device.last_seen_at = Set(now);
    device.update(&state.db).await?;

    // 返回用户全局快照
    if let Some(snapshot) = find_user_snapshot(&state.db, user_id, &data_type).await? {
        Ok(Json(PullResponse {
            device_id: params.device_id,
            data_type,
            data: snapshot.data_payload,
            updated_at: snapshot.created_at.to_rfc3339(),
        }))
    } else {
        Ok(Json(PullResponse {
            device_id: params.device_id,
            data_type,
            data: String::new(),
            updated_at: String::new(),
        }))
    }
}

// ============ 辅助函数 ============

/// 查找用户最新快照（按 user_id 关联的设备）
async fn find_user_snapshot(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> Result<Option<crate::db::schema::device_snapshot::Model>> {
    // 查找该用户的所有设备
    let user_devices = crate::db::schema::Devices::find()
        .filter(crate::db::schema::device::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    if user_devices.is_empty() {
        return Ok(None);
    }

    let device_ids: Vec<i64> = user_devices.iter().map(|d| d.id).collect();

    // 查找这些设备的最新快照
    let snapshot = DeviceSnapshots::find()
        .filter(SnapshotColumn::DeviceId.is_in(device_ids))
        .filter(SnapshotColumn::DataType.eq(data_type))
        .order_by_desc(crate::db::schema::device_snapshot::Column::CreatedAt)
        .one(db)
        .await?;

    Ok(snapshot)
}

/// 删除用户旧快照
async fn delete_user_snapshots(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> Result<()> {
    // 先找到该用户的所有设备
    let user_devices = crate::db::schema::Devices::find()
        .filter(crate::db::schema::device::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let device_ids: Vec<i64> = user_devices.iter().map(|d| d.id).collect();

    // 删除这些设备的旧快照
    for device_id in device_ids {
        let old_snapshots = DeviceSnapshots::find()
            .filter(SnapshotColumn::DeviceId.eq(device_id))
            .filter(SnapshotColumn::DataType.eq(data_type))
            .all(db)
            .await?;

        for old in old_snapshots {
            let model: crate::db::schema::device_snapshot::ActiveModel = old.into();
            model.delete(db).await?;
        }
    }
    Ok(())
}

/// 按标题合并数据：服务器端优先
fn merge_data(server: SyncData, client: SyncData, data_type: &str) -> SyncData {
    match data_type {
        "todos" => {
            // 按标题去重，服务器端优先
            let mut titles: HashMap<String, TodoItem> = HashMap::new();

            // 先加入服务器数据（优先级高）
            for todo in server.todos {
                titles.insert(todo.title.clone(), todo);
            }

            // 再加入客户端数据（如果标题不存在）
            for todo in client.todos {
                titles.entry(todo.title.clone()).or_insert(todo);
            }

            SyncData {
                todos: titles.into_values().collect(),
                ..Default::default()
            }
        }
        "tags" => {
            // 标签：合并去重
            let mut tags: std::collections::HashSet<String> = std::collections::HashSet::new();
            tags.extend(server.tags);
            tags.extend(client.tags);
            SyncData {
                tags: tags.into_iter().collect(),
                ..Default::default()
            }
        }
        "skills" => {
            // 技能：合并去重
            let mut skills: std::collections::HashSet<String> = std::collections::HashSet::new();
            skills.extend(server.skills);
            skills.extend(client.skills);
            SyncData {
                skills: skills.into_iter().collect(),
                ..Default::default()
            }
        }
        _ => client,
    }
}
