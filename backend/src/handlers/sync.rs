//! 同步核心处理模块
//! 采用服务器端合并策略：按标题合并，数据按用户存储
//! 请求和响应都使用 YAML 格式

use crate::db::schema::UserSnapshots;
use crate::db::schema::user_snapshot::Column as SnapshotColumn;
use crate::error::AppError;
use crate::state::AppState;
use crate::services::auth_service::Claims;
use axum::{
    extract::{Query, State, Extension},
    response::IntoResponse,
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
    Overwrite,
    Skip,
    Rename,
}

impl Default for ConflictMode {
    fn default() -> Self {
        ConflictMode::Overwrite
    }
}

/// Push 请求（无需 device_id，从 token 自动获取）
#[derive(Debug, Deserialize)]
pub struct PushRequest {
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
    pub preview: bool,
    pub conflict_mode: String,
    pub merged_data: String,
    pub conflicts: Vec<ConflictPreview>,
    pub summary: PreviewSummary,
}

/// 单个冲突的预览信息
#[derive(Debug, Serialize)]
pub struct ConflictPreview {
    pub title: String,
    pub action: String,
    pub server_item: Option<Box<TodoItem>>,
    pub client_item: TodoItem,
    pub new_title: Option<String>,
}

/// 预览统计摘要
#[derive(Debug, Serialize)]
pub struct PreviewSummary {
    pub total_client_items: usize,
    pub new_items: usize,
    pub overwritten: usize,
    pub skipped: usize,
    pub renamed: usize,
    pub final_total: usize,
}

/// Pull 请求
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub data_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub data_type: String,
    pub data: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub last_sync_at: String,
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TodoItem {
    pub title: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub executor: Option<String>,
    #[serde(default)]
    pub scheduler_enabled: Option<bool>,
    #[serde(default)]
    pub scheduler_config: Option<String>,
    #[serde(default)]
    pub tag_names: Vec<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub worktree: Option<String>,
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
    pub version: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub todos: Vec<TodoItem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

// ============ 处理函数 ============

/// 获取同步状态
pub async fn status(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PullRequest>,
) -> Result<impl IntoResponse, AppError> {
    let data_type = params.data_type.unwrap_or_else(|| "todos".to_string());
    let user_id = claims.sub;

    let latest = find_user_snapshot(&state.db, user_id, &data_type).await;

    let last_sync_at = latest
        .map(|s| s.created_at.to_rfc3339())
        .unwrap_or_default();

    let response = SyncStatus { last_sync_at };

    let yaml = serde_yaml::to_string(&response).unwrap_or_default();
    Ok((
        [("Content-Type", "text/yaml; charset=utf-8")],
        yaml,
    ))
}

/// Push 数据到服务器
pub async fn push(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    let req: PushRequest = serde_yaml::from_str(&body)
        .map_err(|e| AppError::BadRequest(format!("YAML解析失败: {}", e)))?;

    let conflict_mode = req.conflict_mode.unwrap_or_default();
    let dry_run = req.dry_run.unwrap_or(false);
    let user_id = claims.sub;

    tracing::info!(
        "Push 请求: user_id={}, data_type={}, data_len={}, conflict_mode={:?}, dry_run={}",
        user_id,
        req.data_type,
        req.data.len(),
        conflict_mode,
        dry_run
    );

    let now = Utc::now();

    let client_data: SyncData = serde_yaml::from_str(&req.data)
        .map_err(|e| AppError::BadRequest(format!("数据YAML解析失败: {}", e)))?;

    let server_data = if let Some(snapshot) = find_user_snapshot(&state.db, user_id, &req.data_type).await {
        serde_yaml::from_str(&snapshot.data_payload).unwrap_or_default()
    } else {
        SyncData::default()
    };

    // Dry run 模式
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

    // 删除旧快照
    delete_user_snapshots(&state.db, user_id, &req.data_type).await?;

    // 存储新快照
    let new_snapshot = crate::db::schema::user_snapshot::ActiveModel {
        user_id: Set(user_id),
        data_type: Set(req.data_type.clone()),
        data_payload: Set(merged_yaml.clone()),
        checksum: Set(String::new()),
        created_at: Set(now),
        ..Default::default()
    };
    new_snapshot.insert(&state.db).await?;

    // 记录同步日志（含数据摘要）
    let todo_titles: Vec<&str> = merged.todos.iter().map(|t| t.title.as_str()).collect();
    let title_summary = if todo_titles.len() <= 3 {
        todo_titles.join(", ")
    } else {
        format!("{} 等{}项", todo_titles[..3].join(", "), todo_titles.len())
    };
    let detail_summary = format!(
        "data_type={}, todos={}, tags={}, skills={} | {}",
        req.data_type,
        merged.todos.len(),
        merged.tags.len(),
        merged.skills.len(),
        title_summary,
    );

    let sync_log = crate::db::schema::sync_log::ActiveModel {
        user_id: Set(user_id),
        action: Set("push".to_string()),
        status: Set("success".to_string()),
        details: Set(Some(detail_summary)),
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

/// Pull 数据从服务器
pub async fn pull(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PullRequest>,
) -> Result<impl IntoResponse, AppError> {
    let data_type = params.data_type.unwrap_or_else(|| "todos".to_string());
    let user_id = claims.sub;

    tracing::info!("Pull 请求: user_id={}, data_type={}", user_id, data_type);

    let (data, updated_at) = if let Some(snapshot) = find_user_snapshot(&state.db, user_id, &data_type).await {
        (snapshot.data_payload, snapshot.created_at.to_rfc3339())
    } else {
        (String::new(), String::new())
    };

    let response = PullResponse {
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

/// 查找用户最新快照
async fn find_user_snapshot(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> Option<crate::db::schema::user_snapshot::Model> {
    UserSnapshots::find()
        .filter(SnapshotColumn::UserId.eq(user_id))
        .filter(SnapshotColumn::DataType.eq(data_type))
        .order_by_desc(crate::db::schema::user_snapshot::Column::CreatedAt)
        .one(db)
        .await
        .ok()
        .flatten()
}

/// 删除用户快照
async fn delete_user_snapshots(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> Result<(), AppError> {
    let snapshots = UserSnapshots::find()
        .filter(SnapshotColumn::UserId.eq(user_id))
        .filter(SnapshotColumn::DataType.eq(data_type))
        .all(db)
        .await?;

    for old in snapshots {
        let model: crate::db::schema::user_snapshot::ActiveModel = old.into();
        model.delete(db).await?;
    }
    Ok(())
}

/// 合并数据
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
                        ConflictMode::Overwrite => {
                            if let Some(pos) = merged_todos.iter().position(|t| t.title == client_todo.title) {
                                merged_todos[pos] = client_todo.clone();
                            }
                        }
                        ConflictMode::Skip => {}
                        ConflictMode::Rename => {
                            let new_title = generate_unique_title(&client_todo.title, &used_titles);
                            let mut renamed = client_todo.clone();
                            renamed.title = new_title.clone();
                            used_titles.insert(new_title);
                            merged_todos.push(renamed);
                        }
                    }
                } else {
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

/// 带预览的合并
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
                    merged_todos.push(client_todo.clone());
                    used_titles.insert(client_todo.title.clone());
                }
            }

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

fn merge_string_vecs(server: Vec<String>, client: Vec<String>) -> Vec<String> {
    let mut set: std::collections::HashSet<String> = std::collections::HashSet::new();
    set.extend(server.clone());
    set.extend(client.clone());
    set.into_iter().collect()
}