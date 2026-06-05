//! 同步核心处理模块
//! 同步时直接解析 YAML 并存入分立表（user_todos, user_tags, user_skills）
//! 不存储快照，请求和响应都使用 YAML 格式

use crate::db::schema::{UserTodos, UserTags, UserSkills};
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
use std::collections::HashSet;
use std::sync::Arc;

// ============ 请求/响应结构 ============

/// Push 请求
#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub data_type: String,
    pub data: String,
    /// 冲突解决模式：overwrite(默认) | skip | rename
    #[serde(default)]
    pub conflict_mode: Option<String>,
    /// Dry run 模式
    #[serde(default)]
    pub dry_run: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub success: bool,
    pub merged_data: String,
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

    let latest = find_latest_time(&state.db, user_id, &data_type).await;
    let last_sync_at = latest.unwrap_or_default();

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

    let conflict_mode = req.conflict_mode.unwrap_or_else(|| "overwrite".to_string());
    let dry_run = req.dry_run.unwrap_or(false);
    let user_id = claims.sub;

    tracing::info!(
        "Push 请求: user_id={}, data_type={}, conflict_mode={}, dry_run={}",
        user_id, req.data_type, conflict_mode, dry_run
    );

    let now = Utc::now();

    let client_data: SyncData = serde_yaml::from_str(&req.data)
        .map_err(|e| AppError::BadRequest(format!("数据YAML解析失败: {}", e)))?;

    // Dry run 模式 - 返回预览
    if dry_run {
        let server_data = load_user_data(&state.db, user_id, &req.data_type).await;
        let merged = merge_data(server_data, client_data, &req.data_type, &conflict_mode);
        let merged_yaml = serde_yaml::to_string(&merged)
            .map_err(|e| AppError::Internal(format!("序列化失败: {}", e)))?;

        return Ok((
            [("Content-Type", "text/yaml; charset=utf-8")],
            serde_yaml::to_string(&serde_json::json!({
                "success": true,
                "preview": true,
                "merged_data": merged_yaml,
            })).unwrap_or_default(),
        ));
    }

    // 实际同步：直接存入分立表
    save_user_data(&state.db, user_id, &req.data_type, &client_data, &conflict_mode, now).await?;

    // 返回当前所有数据
    let server_data = load_user_data(&state.db, user_id, &req.data_type).await;
    let merged_yaml = serde_yaml::to_string(&server_data)
        .map_err(|e| AppError::Internal(format!("序列化失败: {}", e)))?;

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

    let data = load_user_data_yaml(&state.db, user_id, &data_type).await;
    let updated_at = find_latest_time(&state.db, user_id, &data_type).await.unwrap_or_default();

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

// ============ 数据库操作 ============

/// 加载用户数据
async fn load_user_data(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> SyncData {
    match data_type {
        "todos" => {
            let todos = UserTodos::find()
                .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
                .all(db)
                .await
                .unwrap_or_default();
            SyncData {
                todos: todos
                    .into_iter()
                    .map(|t| TodoItem {
                        title: t.title,
                        prompt: t.prompt,
                        status: t.status,
                        executor: t.executor,
                        scheduler_enabled: t.scheduler_enabled.map(|v| v == 1),
                        scheduler_config: t.scheduler_config,
                        tag_names: t.tag_names.and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_default(),
                        workspace: t.workspace,
                        worktree: t.worktree,
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }
        }
        "tags" => {
            let tags = UserTags::find()
                .filter(crate::db::schema::user_tag::Column::UserId.eq(user_id))
                .all(db)
                .await
                .unwrap_or_default();
            SyncData {
                tags: tags.into_iter().map(|t| t.name).collect(),
                ..Default::default()
            }
        }
        "skills" => {
            let skills = UserSkills::find()
                .filter(crate::db::schema::user_skill::Column::UserId.eq(user_id))
                .all(db)
                .await
                .unwrap_or_default();
            SyncData {
                skills: skills.into_iter().map(|s| s.name).collect(),
                ..Default::default()
            }
        }
        _ => SyncData::default(),
    }
}

/// 加载用户数据为 YAML 字符串
async fn load_user_data_yaml(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> String {
    let data = load_user_data(db, user_id, data_type).await;
    serde_yaml::to_string(&data).unwrap_or_default()
}

/// 保存用户数据（合并模式）
async fn save_user_data(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
    data: &SyncData,
    conflict_mode: &str,
    now: chrono::DateTime<Utc>,
) -> Result<(), AppError> {
    match data_type {
        "todos" => {
            // 获取现有数据
            let existing = UserTodos::find()
                .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
                .all(db)
                .await?;
            let mut existing_map: std::collections::HashMap<String, _> = existing
                .into_iter()
                .map(|t| (t.title.clone(), t))
                .collect();

            for todo in &data.todos {
                if let Some(existing) = existing_map.get(&todo.title) {
                    match conflict_mode {
                        "skip" => continue,
                        "rename" => {
                            // 生成新标题
                            let mut counter = 1;
                            let mut new_title = format!("{} ({})", todo.title, counter);
                            while existing_map.contains_key(&new_title) {
                                counter += 1;
                                new_title = format!("{} ({})", todo.title, counter);
                            }
                            insert_todo(db, user_id, todo, &new_title, now).await?;
                        }
                        _ => {
                            // overwrite: 更新现有记录
                            update_todo(db, &existing.id, todo, now).await?;
                        }
                    }
                } else {
                    insert_todo(db, user_id, todo, &todo.title, now).await?;
                }
                existing_map.remove(&todo.title);
            }
        }
        "tags" => {
            for tag in &data.tags {
                let existing = UserTags::find()
                    .filter(crate::db::schema::user_tag::Column::UserId.eq(user_id))
                    .filter(crate::db::schema::user_tag::Column::Name.eq(tag))
                    .one(db)
                    .await?;
                if existing.is_none() {
                    let model = crate::db::schema::user_tag::ActiveModel {
                        user_id: Set(user_id),
                        name: Set(tag.clone()),
                        created_at: Set(now),
                        ..Default::default()
                    };
                    model.insert(db).await?;
                }
            }
        }
        "skills" => {
            for skill in &data.skills {
                let existing = UserSkills::find()
                    .filter(crate::db::schema::user_skill::Column::UserId.eq(user_id))
                    .filter(crate::db::schema::user_skill::Column::Name.eq(skill))
                    .one(db)
                    .await?;
                if existing.is_none() {
                    let model = crate::db::schema::user_skill::ActiveModel {
                        user_id: Set(user_id),
                        name: Set(skill.clone()),
                        created_at: Set(now),
                        ..Default::default()
                    };
                    model.insert(db).await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn insert_todo(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    todo: &TodoItem,
    title: &str,
    now: chrono::DateTime<Utc>,
) -> Result<(), AppError> {
    let tag_names_json = serde_json::to_string(&todo.tag_names).unwrap_or_default();
    let sched_enabled = todo.scheduler_enabled.unwrap_or(false);

    let model = crate::db::schema::user_todo::ActiveModel {
        user_id: Set(user_id),
        title: Set(title.to_string()),
        prompt: Set(todo.prompt.clone()),
        status: Set(todo.status.clone()),
        executor: Set(todo.executor.clone()),
        scheduler_enabled: Set(Some(if sched_enabled { 1 } else { 0 })),
        scheduler_config: Set(todo.scheduler_config.clone()),
        tag_names: Set(Some(tag_names_json)),
        workspace: Set(todo.workspace.clone()),
        worktree: Set(todo.worktree.clone()),
        created_at: Set(now),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    model.insert(db).await?;
    Ok(())
}

async fn update_todo(
    db: &sea_orm::DatabaseConnection,
    id: &i64,
    todo: &TodoItem,
    now: chrono::DateTime<Utc>,
) -> Result<(), AppError> {
    let existing = UserTodos::find()
        .filter(crate::db::schema::user_todo::Column::Id.eq(*id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("Todo 不存在".to_string()))?;

    let mut model: crate::db::schema::user_todo::ActiveModel = existing.into();
    let tag_names_json = serde_json::to_string(&todo.tag_names).unwrap_or_default();
    let sched_enabled = todo.scheduler_enabled.unwrap_or(false);

    model.prompt = Set(todo.prompt.clone());
    model.status = Set(todo.status.clone());
    model.executor = Set(todo.executor.clone());
    model.scheduler_enabled = Set(Some(if sched_enabled { 1 } else { 0 }));
    model.scheduler_config = Set(todo.scheduler_config.clone());
    model.tag_names = Set(Some(tag_names_json));
    model.workspace = Set(todo.workspace.clone());
    model.worktree = Set(todo.worktree.clone());
    model.updated_at = Set(Some(now));

    model.update(db).await?;
    Ok(())
}

/// 查找最新更新时间
async fn find_latest_time(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    data_type: &str,
) -> Option<String> {
    match data_type {
        "todos" => UserTodos::find()
            .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
            .order_by_desc(crate::db::schema::user_todo::Column::CreatedAt)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|t| t.created_at.to_rfc3339()),
        "tags" => UserTags::find()
            .filter(crate::db::schema::user_tag::Column::UserId.eq(user_id))
            .order_by_desc(crate::db::schema::user_tag::Column::CreatedAt)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|t| t.created_at.to_rfc3339()),
        "skills" => UserSkills::find()
            .filter(crate::db::schema::user_skill::Column::UserId.eq(user_id))
            .order_by_desc(crate::db::schema::user_skill::Column::CreatedAt)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|s| s.created_at.to_rfc3339()),
        _ => None,
    }
}

// ============ 合并逻辑 ============

fn merge_data(server: SyncData, client: SyncData, data_type: &str, mode: &str) -> SyncData {
    match data_type {
        "todos" => {
            let mut server_titles: std::collections::HashMap<String, TodoItem> = std::collections::HashMap::new();
            for todo in &server.todos {
                server_titles.insert(todo.title.clone(), todo.clone());
            }

            let mut merged_todos = server.todos.clone();
            let mut used_titles: HashSet<String> = server_titles.keys().cloned().collect();

            for client_todo in &client.todos {
                if server_titles.contains_key(&client_todo.title) {
                    match mode {
                        "overwrite" => {
                            if let Some(pos) = merged_todos.iter().position(|t| t.title == client_todo.title) {
                                merged_todos[pos] = client_todo.clone();
                            }
                        }
                        "skip" => {}
                        "rename" => {
                            let new_title = generate_unique_title(&client_todo.title, &used_titles);
                            let mut renamed = client_todo.clone();
                            renamed.title = new_title.clone();
                            used_titles.insert(new_title);
                            merged_todos.push(renamed);
                        }
                        _ => {}
                    }
                } else {
                    merged_todos.push(client_todo.clone());
                    used_titles.insert(client_todo.title.clone());
                }
            }

            SyncData {
                version: Some("1.0".to_string()),
                todos: merged_todos,
                ..Default::default()
            }
        }
        "tags" => SyncData {
            version: Some("1.0".to_string()),
            tags: merge_string_vec(server.tags, client.tags),
            ..Default::default()
        },
        "skills" => SyncData {
            version: Some("1.0".to_string()),
            skills: merge_string_vec(server.skills, client.skills),
            ..Default::default()
        },
        _ => client,
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

fn merge_string_vec(server: Vec<String>, client: Vec<String>) -> Vec<String> {
    let mut set: std::collections::HashSet<String> = std::collections::HashSet::new();
    set.extend(server);
    set.extend(client);
    set.into_iter().collect()
}
