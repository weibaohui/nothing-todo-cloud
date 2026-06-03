//! 同步核心处理模块
//! 采用服务器端合并策略：按标题合并，所有设备共享用户级全局快照
//! 请求和响应都使用 YAML 格式

use crate::db::schema::DeviceSnapshots;
use crate::db::schema::device_snapshot::Column as SnapshotColumn;
use crate::db::schema::device::Column as DeviceColumn;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============ 请求/响应结构 ============

#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub device_id: i64,
    pub data_type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub success: bool,
    pub merged_data: String,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub device_id: i64,
    pub data_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub device_id: i64,
    pub data_type: String,
    pub data: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub device_id: i64,
    pub last_sync_at: String,
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TodoItem {
    pub title: String,
    #[serde(default)]
    pub done: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SyncData {
    #[serde(default)]
    pub todos: Vec<TodoItem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

// ============ 处理函数 ============

pub async fn status(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullRequest>,
) -> Result<impl IntoResponse, AppError> {
    let data_type = params.data_type.unwrap_or_else(|| "todos".to_string());

    let device = crate::db::schema::Devices::find_by_id(params.device_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_string()))?;

    let latest = find_user_snapshot(&state.db, device.user_id, &data_type).await;

    let last_sync_at = latest
        .map(|s| s.created_at.to_rfc3339())
        .unwrap_or_default();

    let response = SyncStatus {
        device_id: params.device_id,
        last_sync_at,
    };

    let yaml = serde_yaml::to_string(&response).unwrap_or_default();
    Ok((
        [("Content-Type", "text/yaml; charset=utf-8")],
        yaml,
    ))
}

pub async fn push(
    State(state): State<Arc<AppState>>,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    let req: PushRequest = serde_yaml::from_str(&body)
        .map_err(|e| AppError::BadRequest(format!("YAML解析失败: {}", e)))?;

    tracing::info!(
        "Push 请求: device_id={}, data_type={}, data_len={}",
        req.device_id,
        req.data_type,
        req.data.len()
    );

    let now = Utc::now();

    let device = crate::db::schema::Devices::find_by_id(req.device_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_string()))?;
    let user_id = device.user_id;

    let client_data: SyncData = serde_yaml::from_str(&req.data)
        .map_err(|e| AppError::BadRequest(format!("数据YAML解析失败: {}", e)))?;

    let server_data = if let Some(snapshot) = find_user_snapshot(&state.db, user_id, &req.data_type).await {
        serde_yaml::from_str(&snapshot.data_payload).unwrap_or_default()
    } else {
        SyncData::default()
    };

    let merged = merge_data(server_data, client_data, &req.data_type);

    let merged_yaml = serde_yaml::to_string(&merged)
        .map_err(|e| AppError::Internal(format!("YAML序列化失败: {}", e)))?;

    delete_user_snapshots(&state.db, user_id, &req.data_type).await?;

    let new_snapshot = crate::db::schema::device_snapshot::ActiveModel {
        device_id: Set(req.device_id),
        version: Set(1),
        data_type: Set(req.data_type.clone()),
        data_payload: Set(merged_yaml.clone()),
        checksum: Set(String::new()),
        created_at: Set(now),
        ..Default::default()
    };
    new_snapshot.insert(&state.db).await?;

    let sync_log = crate::db::schema::sync_log::ActiveModel {
        device_id: Set(req.device_id),
        action: Set("push".to_string()),
        status: Set("success".to_string()),
        details: Set(Some(format!("data_type={}", req.data_type))),
        created_at: Set(now),
        ..Default::default()
    };
    sync_log.insert(&state.db).await?;

    let response = PushResponse {
        success: true,
        merged_data: merged_yaml,
    };

    let yaml = serde_yaml::to_string(&response).unwrap_or_default();
    Ok((
        [("Content-Type", "text/yaml; charset=utf-8")],
        yaml,
    ))
}

pub async fn pull(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullRequest>,
) -> Result<impl IntoResponse, AppError> {
    let data_type = params.data_type.unwrap_or_else(|| "todos".to_string());

    tracing::info!("Pull 请求: device_id={}, data_type={}", params.device_id, data_type);

    let device = crate::db::schema::Devices::find_by_id(params.device_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_string()))?;
    let user_id = device.user_id;

    let now = Utc::now();
    let mut device: crate::db::schema::device::ActiveModel = device.into();
    device.last_seen_at = Set(now);
    device.update(&state.db).await?;

    let (data, updated_at) = if let Some(snapshot) = find_user_snapshot(&state.db, user_id, &data_type).await {
        (snapshot.data_payload, snapshot.created_at.to_rfc3339())
    } else {
        (String::new(), String::new())
    };

    let response = PullResponse {
        device_id: params.device_id,
        data_type,
        data,
        updated_at,
    };

    let yaml = serde_yaml::to_string(&response).unwrap_or_default();
    Ok((
        [("Content-Type", "text/yaml; charset=utf-8")],
        yaml,
    ))
}

// ============ 辅助函数 ============

async fn find_user_snapshot(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> Option<crate::db::schema::device_snapshot::Model> {
    let user_devices = crate::db::schema::Devices::find()
        .filter(DeviceColumn::UserId.eq(user_id))
        .all(db)
        .await
        .ok()?;

    if user_devices.is_empty() {
        return None;
    }

    let device_ids: Vec<i64> = user_devices.iter().map(|d| d.id).collect();

    DeviceSnapshots::find()
        .filter(SnapshotColumn::DeviceId.is_in(device_ids))
        .filter(SnapshotColumn::DataType.eq(data_type))
        .order_by_desc(crate::db::schema::device_snapshot::Column::CreatedAt)
        .one(db)
        .await
        .ok()
        .flatten()
}

async fn delete_user_snapshots(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> Result<(), AppError> {
    let user_devices = crate::db::schema::Devices::find()
        .filter(DeviceColumn::UserId.eq(user_id))
        .all(db)
        .await?;

    for device in user_devices {
        let old_snapshots = DeviceSnapshots::find()
            .filter(SnapshotColumn::DeviceId.eq(device.id))
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

fn merge_data(server: SyncData, client: SyncData, data_type: &str) -> SyncData {
    match data_type {
        "todos" => {
            let mut titles: HashMap<String, TodoItem> = HashMap::new();
            for todo in server.todos {
                titles.insert(todo.title.clone(), todo);
            }
            for todo in client.todos {
                titles.entry(todo.title.clone()).or_insert(todo);
            }
            SyncData {
                todos: titles.into_values().collect(),
                ..Default::default()
            }
        }
        "tags" => {
            let mut tags: std::collections::HashSet<String> = std::collections::HashSet::new();
            tags.extend(server.tags);
            tags.extend(client.tags);
            SyncData {
                tags: tags.into_iter().collect(),
                ..Default::default()
            }
        }
        "skills" => {
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
