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
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// ============ 请求/响应结构 ============

/// 冲突解决模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictMode {
    /// 覆盖：客户端数据覆盖服务端数据
    Overwrite,
    /// 跳过：保留服务端数据，忽略客户端冲突项
    Skip,
    /// 重命名：保留双方，客户端冲突项添加后缀
    Rename,
}

impl Default for ConflictMode {
    fn default() -> Self {
        ConflictMode::Overwrite
    }
}

#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub device_id: i64,
    pub data_type: String,
    pub data: String,
    /// 冲突解决模式：overwrite(默认) | skip | rename
    #[serde(default)]
    pub conflict_mode: Option<ConflictMode>,
    /// Dry run 模式：不实际执行，只返回预览结果
    #[serde(default)]
    pub dry_run: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub success: bool,
    pub merged_data: String,
}

/// Dry run 预览响应
#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    pub success: bool,
    /// 预览模式标识
    pub preview: bool,
    /// 冲突解决模式
    pub conflict_mode: String,
    /// 合并后的数据预览
    pub merged_data: String,
    /// 冲突详情列表
    pub conflicts: Vec<ConflictPreview>,
    /// 统计信息
    pub summary: PreviewSummary,
}

/// 单个冲突的预览信息
#[derive(Debug, Serialize)]
pub struct ConflictPreview {
    /// 标题
    pub title: String,
    /// 冲突类型
    pub action: String,  // "overwrite" | "skip" | "rename"
    /// 服务端原始项（如果存在）
    pub server_item: Option<Box<TodoItem>>,
    /// 客户端提交的项
    pub client_item: TodoItem,
    /// 如果是 rename，列出新标题
    pub new_title: Option<String>,
}

/// 预览统计摘要
#[derive(Debug, Serialize)]
pub struct PreviewSummary {
    pub total_client_items: usize,
    pub new_items: usize,       // 客户端新增（服务端无此标题）
    pub overwritten: usize,      // 被覆盖的
    pub skipped: usize,         // 被跳过的
    pub renamed: usize,         // 被重命名的
    pub final_total: usize,      // 最终总数
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
// 与 nothing-todo 本地备份格式保持一致 (todo-backup-*.zip)

/// Todo 项结构，兼容本地备份格式
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TodoItem {
    pub title: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub status: Option<String>,           // "completed" | "pending"
    #[serde(default)]
    pub executor: Option<String>,
    #[serde(default)]
    pub scheduler_enabled: Option<bool>,
    #[serde(default)]
    pub scheduler_config: Option<String>, // cron 表达式
    #[serde(default)]
    pub tag_names: Vec<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub worktree: Option<String>,
    // 兼容旧字段
    #[serde(default)]
    pub done: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// 同步数据结构，与备份文件格式一致
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SyncData {
    #[serde(default)]
    pub version: Option<String>,   // 备份格式版本
    #[serde(default)]
    pub created_at: Option<String>, // 备份创建时间
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

    let conflict_mode = req.conflict_mode.unwrap_or_default();
    let dry_run = req.dry_run.unwrap_or(false);
    tracing::info!(
        "Push 请求: device_id={}, data_type={}, data_len={}, conflict_mode={:?}, dry_run={}",
        req.device_id,
        req.data_type,
        req.data.len(),
        conflict_mode,
        dry_run
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

    // Dry run 模式：只返回预览，不实际执行
    if dry_run {
        let (merged, conflicts) = merge_data_with_preview(server_data, client_data.clone(), &req.data_type, conflict_mode);
        let merged_yaml = serde_yaml::to_string(&merged)
            .map_err(|e| AppError::Internal(format!("YAML序列化失败: {}", e)))?;

        let total_client_items = client_data.todos.len();
        let new_items = conflicts.iter().filter(|c| c.action != "skip").count();
        let overwritten = conflicts.iter().filter(|c| c.action == "overwrite").count();
        let skipped = conflicts.iter().filter(|c| c.action == "skip").count();
        let renamed = conflicts.iter().filter(|c| c.action == "rename").count();

        let preview_response = PreviewResponse {
            success: true,
            preview: true,
            conflict_mode: format!("{:?}", conflict_mode).to_lowercase(),
            merged_data: merged_yaml,
            conflicts,
            summary: PreviewSummary {
                total_client_items,
                new_items,
                overwritten,
                skipped,
                renamed,
                final_total: merged.todos.len(),
            },
        };

        let yaml = serde_yaml::to_string(&preview_response)
            .map_err(|e| AppError::Internal(format!("YAML序列化失败: {}", e)))?;
        return Ok((
            [("Content-Type", "text/yaml; charset=utf-8")],
            yaml,
        ));
    }

    let merged = merge_data(server_data, client_data, &req.data_type, conflict_mode);

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

/// 合并数据：根据冲突模式处理
/// - Overwrite: 客户端数据覆盖服务端数据
/// - Skip: 保留服务端数据，忽略客户端冲突项
/// - Rename: 保留双方，客户端冲突项添加后缀
fn merge_data(server: SyncData, client: SyncData, data_type: &str, mode: ConflictMode) -> SyncData {
    match data_type {
        "todos" => {
            let mut server_titles: HashMap<String, TodoItem> = HashMap::new();
            for todo in &server.todos {
                server_titles.insert(todo.title.clone(), todo.clone());
            }

            let mut merged_todos: Vec<TodoItem> = server.todos.clone();
            let mut used_titles: HashSet<String> = server_titles.keys().cloned().collect();

            for client_todo in &client.todos {
                if server_titles.contains_key(&client_todo.title) {
                    match mode {
                        // 覆盖：客户端替换服务端
                        ConflictMode::Overwrite => {
                            if let Some(pos) = merged_todos.iter().position(|t| t.title == client_todo.title) {
                                merged_todos[pos] = client_todo.clone();
                            }
                        }
                        // 跳过：保留服务端，忽略客户端
                        ConflictMode::Skip => {
                            // 什么都不做，保留 server 的
                        }
                        // 重命名：给客户端添加后缀
                        ConflictMode::Rename => {
                            let new_title = generate_unique_title(&client_todo.title, &used_titles);
                            let mut renamed = client_todo.clone();
                            renamed.title = new_title.clone();
                            used_titles.insert(new_title);
                            merged_todos.push(renamed);
                        }
                    }
                } else {
                    // 标题不存在，直接添加
                    merged_todos.push(client_todo.clone());
                    used_titles.insert(client_todo.title.clone());
                }
            }

            SyncData {
                version: server.version.or(client.version).or(Some("1.0".to_string())),
                created_at: Some(Utc::now().to_rfc3339()),
                todos: merged_todos,
                tags: merge_string_vecs(server.tags, client.tags),
                skills: merge_string_vecs(server.skills, client.skills),
            }
        }
        "tags" => {
            SyncData {
                version: server.version.or(client.version).or(Some("1.0".to_string())),
                created_at: Some(Utc::now().to_rfc3339()),
                todos: vec![],
                tags: merge_string_vecs(server.tags, client.tags),
                skills: vec![],
            }
        }
        "skills" => {
            SyncData {
                version: server.version.or(client.version).or(Some("1.0".to_string())),
                created_at: Some(Utc::now().to_rfc3339()),
                todos: vec![],
                tags: vec![],
                skills: merge_string_vecs(server.skills, client.skills),
            }
        }
        _ => client,
    }
}

/// 合并数据并生成冲突预览（用于 dry run）
/// 返回 (合并后的数据, 冲突详情列表)
fn merge_data_with_preview(
    server: SyncData,
    client: SyncData,
    data_type: &str,
    mode: ConflictMode,
) -> (SyncData, Vec<ConflictPreview>) {
    let mut conflicts = Vec::new();

    match data_type {
        "todos" => {
            let mut server_titles: HashMap<String, TodoItem> = HashMap::new();
            for todo in &server.todos {
                server_titles.insert(todo.title.clone(), todo.clone());
            }

            let mut merged_todos: Vec<TodoItem> = server.todos.clone();
            let mut used_titles: HashSet<String> = server_titles.keys().cloned().collect();

            for client_todo in &client.todos {
                if server_titles.contains_key(&client_todo.title) {
                    let server_item = server_titles.get(&client_todo.title).cloned();
                    match mode {
                        ConflictMode::Overwrite => {
                            conflicts.push(ConflictPreview {
                                title: client_todo.title.clone(),
                                action: "overwrite".to_string(),
                                server_item: server_item.map(Box::new),
                                client_item: client_todo.clone(),
                                new_title: None,
                            });
                            if let Some(pos) = merged_todos.iter().position(|t| t.title == client_todo.title) {
                                merged_todos[pos] = client_todo.clone();
                            }
                        }
                        ConflictMode::Skip => {
                            conflicts.push(ConflictPreview {
                                title: client_todo.title.clone(),
                                action: "skip".to_string(),
                                server_item: server_item.map(Box::new),
                                client_item: client_todo.clone(),
                                new_title: None,
                            });
                            // 什么都不做，保留 server 的
                        }
                        ConflictMode::Rename => {
                            let new_title = generate_unique_title(&client_todo.title, &used_titles);
                            conflicts.push(ConflictPreview {
                                title: client_todo.title.clone(),
                                action: "rename".to_string(),
                                server_item: server_item.map(Box::new),
                                client_item: client_todo.clone(),
                                new_title: Some(new_title.clone()),
                            });
                            let mut renamed = client_todo.clone();
                            renamed.title = new_title.clone();
                            used_titles.insert(new_title);
                            merged_todos.push(renamed);
                        }
                    }
                } else {
                    // 标题不存在，直接添加
                    merged_todos.push(client_todo.clone());
                    used_titles.insert(client_todo.title.clone());
                }
            }

            let _final_total = merged_todos.len();
            let _new_items = client.todos.len() - conflicts.iter().filter(|c| c.action != "skip").count();
            let _overwritten = conflicts.iter().filter(|c| c.action == "overwrite").count();
            let _skipped = conflicts.iter().filter(|c| c.action == "skip").count();
            let _renamed = conflicts.iter().filter(|c| c.action == "rename").count();

            (SyncData {
                version: server.version.or(client.version).or(Some("1.0".to_string())),
                created_at: Some(Utc::now().to_rfc3339()),
                todos: merged_todos,
                tags: merge_string_vecs(server.tags, client.tags),
                skills: merge_string_vecs(server.skills, client.skills),
            }, conflicts)
        }
        "tags" | "skills" => {
            (SyncData {
                version: server.version.or(client.version).or(Some("1.0".to_string())),
                created_at: Some(Utc::now().to_rfc3339()),
                todos: vec![],
                tags: if data_type == "tags" { merge_string_vecs(server.tags, client.tags) } else { vec![] },
                skills: if data_type == "skills" { merge_string_vecs(server.skills, client.skills) } else { vec![] },
            }, conflicts)
        }
        _ => (client, conflicts),
    }
}

/// 生成唯一的标题（用于 rename 模式）
/// 如果 "买菜" 已存在，尝试 "买菜 (1)", "买菜 (2)" 等
fn generate_unique_title(base_title: &str, used_titles: &HashSet<String>) -> String {
    let mut counter = 1;
    loop {
        let new_title = format!("{} ({})", base_title, counter);
        if !used_titles.contains(&new_title) {
            return new_title;
        }
        counter += 1;
    }
}

/// 合并字符串数组（去重 + 保持顺序）
fn merge_string_vecs(server: Vec<String>, client: Vec<String>) -> Vec<String> {
    let mut set: std::collections::HashSet<String> = std::collections::HashSet::new();
    set.extend(server.clone());
    set.extend(client.clone());
    set.into_iter().collect()
}
