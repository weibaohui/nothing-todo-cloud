//! Todo 独立条目管理模块
//! 每条 Todo 独立存储，支持单独的增删改查操作

use std::sync::Arc;

use crate::db::schema::UserTodos;
use crate::error::AppError;
use crate::state::AppState;
use crate::services::auth_service::Claims;
use axum::{
    extract::{Path, Query, State, Extension},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

/// ============ 请求/响应结构 ============

/// 创建 Todo 请求
#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
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

/// 更新 Todo 请求
#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    #[serde(default)]
    pub title: Option<String>,
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
    pub tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub worktree: Option<String>,
}

/// Todo 响应结构
#[derive(Debug, Serialize)]
pub struct TodoResponse {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub prompt: Option<String>,
    pub status: Option<String>,
    pub executor: Option<String>,
    pub scheduler_enabled: Option<bool>,
    pub scheduler_config: Option<String>,
    pub tag_names: Vec<String>,
    pub workspace: Option<String>,
    pub worktree: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
    pub status: Option<String>,
}

/// ============ 处理函数 ============

/// 获取当前用户的所有 Todo（支持按标签/状态筛选）
pub async fn list_todos(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;

    // 构建查询
    let mut select = UserTodos::find()
        .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
        .order_by_asc(crate::db::schema::user_todo::Column::Title);

    let todos = select.all(&state.db).await?;

    // 过滤标签
    let todos: Vec<TodoResponse> = todos
        .into_iter()
        .filter(|t| {
            // 按标签过滤
            if let Some(ref filter_tag) = query.tag {
                let tags: Vec<String> = t.tag_names.as_ref()
                    .and_then(|v| serde_json::from_str(v).ok())
                    .unwrap_or_default();
                if !tags.contains(filter_tag) {
                    return false;
                }
            }
            // 按状态过滤
            if let Some(ref filter_status) = query.status {
                if t.status.as_ref() != Some(filter_status) {
                    return false;
                }
            }
            true
        })
        .map(|t| {
            let tag_names: Vec<String> = t.tag_names
                .and_then(|v| serde_json::from_str(&v).ok())
                .unwrap_or_default();
            TodoResponse {
                id: t.id,
                user_id: t.user_id,
                title: t.title,
                prompt: t.prompt,
                status: t.status,
                executor: t.executor,
                scheduler_enabled: t.scheduler_enabled.map(|v| v == 1),
                scheduler_config: t.scheduler_config,
                tag_names,
                workspace: t.workspace,
                worktree: t.worktree,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.map(|v| v.to_rfc3339()),
            }
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": todos,
        "total": todos.len()
    })))
}

/// 获取单个 Todo
pub async fn get_todo(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(todo_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;

    let todo = UserTodos::find()
        .filter(crate::db::schema::user_todo::Column::Id.eq(todo_id))
        .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Todo 不存在".to_string()))?;

    let tag_names: Vec<String> = todo.tag_names
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default();

    let response = TodoResponse {
        id: todo.id,
        user_id: todo.user_id,
        title: todo.title,
        prompt: todo.prompt,
        status: todo.status,
        executor: todo.executor,
        scheduler_enabled: todo.scheduler_enabled.map(|v| v == 1),
        scheduler_config: todo.scheduler_config,
        tag_names,
        workspace: todo.workspace,
        worktree: todo.worktree,
        created_at: todo.created_at.to_rfc3339(),
        updated_at: todo.updated_at.map(|v| v.to_rfc3339()),
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}

/// 创建 Todo
pub async fn create_todo(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateTodoRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;
    let now = Utc::now();

    // 检查标题是否已存在
    let existing = UserTodos::find()
        .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
        .filter(crate::db::schema::user_todo::Column::Title.eq(&req.title))
        .one(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::BadRequest("标题已存在".to_string()));
    }

    let tag_names_json = serde_json::to_string(&req.tag_names).unwrap_or_default();
    let sched_enabled = req.scheduler_enabled.unwrap_or(false);

    let model = crate::db::schema::user_todo::ActiveModel {
        user_id: Set(user_id),
        title: Set(req.title),
        prompt: Set(req.prompt),
        status: Set(req.status),
        executor: Set(req.executor),
        scheduler_enabled: Set(Some(if sched_enabled { 1 } else { 0 })),
        scheduler_config: Set(req.scheduler_config),
        tag_names: Set(Some(tag_names_json)),
        workspace: Set(req.workspace),
        worktree: Set(req.worktree),
        created_at: Set(now),
        updated_at: Set(Some(now)),
        ..Default::default()
    };

    let todo = model.insert(&state.db).await?;

    let tag_names: Vec<String> = todo.tag_names
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default();

    let response = TodoResponse {
        id: todo.id,
        user_id: todo.user_id,
        title: todo.title,
        prompt: todo.prompt,
        status: todo.status,
        executor: todo.executor,
        scheduler_enabled: todo.scheduler_enabled.map(|v| v == 1),
        scheduler_config: todo.scheduler_config,
        tag_names,
        workspace: todo.workspace,
        worktree: todo.worktree,
        created_at: todo.created_at.to_rfc3339(),
        updated_at: todo.updated_at.map(|v| v.to_rfc3339()),
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}

/// 更新 Todo
pub async fn update_todo(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(todo_id): Path<i64>,
    Json(req): Json<UpdateTodoRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;
    let now = Utc::now();

    // 查找现有 Todo
    let existing = UserTodos::find()
        .filter(crate::db::schema::user_todo::Column::Id.eq(todo_id))
        .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Todo 不存在".to_string()))?;

    // 如果改标题，检查是否与其他重复
    if let Some(ref new_title) = req.title {
        if new_title != &existing.title {
            let duplicate = UserTodos::find()
                .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
                .filter(crate::db::schema::user_todo::Column::Title.eq(new_title))
                .filter(crate::db::schema::user_todo::Column::Id.ne(todo_id))
                .one(&state.db)
                .await?;
            if duplicate.is_some() {
                return Err(AppError::BadRequest("标题已存在".to_string()));
            }
        }
    }

    let mut model: crate::db::schema::user_todo::ActiveModel = existing.into();

    // 只更新提供的字段
    if let Some(title) = req.title {
        model.title = Set(title);
    }
    if let Some(prompt) = req.prompt {
        model.prompt = Set(Some(prompt));
    }
    if let Some(status) = req.status {
        model.status = Set(Some(status));
    }
    if let Some(executor) = req.executor {
        model.executor = Set(Some(executor));
    }
    if let Some(scheduler_enabled) = req.scheduler_enabled {
        model.scheduler_enabled = Set(Some(if scheduler_enabled { 1 } else { 0 }));
    }
    if let Some(scheduler_config) = req.scheduler_config {
        model.scheduler_config = Set(Some(scheduler_config));
    }
    if let Some(tag_names) = req.tag_names {
        model.tag_names = Set(Some(serde_json::to_string(&tag_names).unwrap_or_default()));
    }
    if let Some(workspace) = req.workspace {
        model.workspace = Set(Some(workspace));
    }
    if let Some(worktree) = req.worktree {
        model.worktree = Set(Some(worktree));
    }
    model.updated_at = Set(Some(now));

    let todo = model.update(&state.db).await?;

    let tag_names: Vec<String> = todo.tag_names
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default();

    let response = TodoResponse {
        id: todo.id,
        user_id: todo.user_id,
        title: todo.title,
        prompt: todo.prompt,
        status: todo.status,
        executor: todo.executor,
        scheduler_enabled: todo.scheduler_enabled.map(|v| v == 1),
        scheduler_config: todo.scheduler_config,
        tag_names,
        workspace: todo.workspace,
        worktree: todo.worktree,
        created_at: todo.created_at.to_rfc3339(),
        updated_at: todo.updated_at.map(|v| v.to_rfc3339()),
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}

/// 删除 Todo
pub async fn delete_todo(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(todo_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;

    // 查找现有 Todo
    let existing = UserTodos::find()
        .filter(crate::db::schema::user_todo::Column::Id.eq(todo_id))
        .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Todo 不存在".to_string()))?;

    let mut model: crate::db::schema::user_todo::ActiveModel = existing.into();
    model.delete(&state.db).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "删除成功"
    })))
}

/// 获取所有标签（用于筛选）
pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;

    let todos = UserTodos::find()
        .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
        .all(&state.db)
        .await?;

    // 收集所有标签
    let mut all_tags: std::collections::HashSet<String> = std::collections::HashSet::new();
    for todo in todos {
        if let Some(tag_json) = todo.tag_names {
            if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tag_json) {
                all_tags.extend(tags);
            }
        }
    }

    let mut tags: Vec<String> = all_tags.into_iter().collect();
    tags.sort();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": tags
    })))
}

/// ============ 导入功能 ============

/// 从 snapshot YAML 导入数据
#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub yaml_data: String,
    /// 导入模式：merge(合并) | replace(替换)
    #[serde(default = "default_import_mode")]
    pub mode: String,
}

fn default_import_mode() -> String {
    "merge".to_string()
}

/// 解析后的导入统计
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub todos_imported: usize,
    pub tags_imported: usize,
    pub skills_imported: usize,
    pub todos_skipped: usize,
}

/// 从 YAML 解析单个 TodoItem
#[derive(Debug, Deserialize)]
struct YamlTodoItem {
    title: String,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    executor: Option<String>,
    #[serde(default)]
    scheduler_enabled: Option<bool>,
    #[serde(default)]
    scheduler_config: Option<String>,
    #[serde(default)]
    tag_names: Vec<String>,
    #[serde(default)]
    workspace: Option<String>,
    #[serde(default)]
    worktree: Option<String>,
}

/// 解析后的 YAML 结构
#[derive(Debug, Deserialize)]
struct YamlSyncData {
    #[serde(default)]
    todos: Vec<YamlTodoItem>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
}

/// 从 snapshot YAML 导入数据到数据库
pub async fn import_data(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ImportRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;
    let now = Utc::now();

    // 解析 YAML
    let yaml_data: YamlSyncData = serde_yaml::from_str(&req.yaml_data)
        .map_err(|e| AppError::BadRequest(format!("YAML 解析失败: {}", e)))?;

    let mut todos_imported = 0;
    let mut todos_skipped = 0;
    let mut tags_imported = 0;
    let mut skills_imported = 0;

    // 如果是 replace 模式，先删除现有数据
    if req.mode == "replace" {
        // 删除现有 todos
        let existing_todos = UserTodos::find()
            .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
            .all(&state.db)
            .await?;
        for t in existing_todos {
            let m: crate::db::schema::user_todo::ActiveModel = t.into();
            m.delete(&state.db).await?;
        }

        // 删除现有 tags
        let existing_tags = crate::db::schema::UserTags::find()
            .filter(crate::db::schema::user_tag::Column::UserId.eq(user_id))
            .all(&state.db)
            .await?;
        for t in existing_tags {
            let m: crate::db::schema::user_tag::ActiveModel = t.into();
            m.delete(&state.db).await?;
        }

        // 删除现有 skills
        let existing_skills = crate::db::schema::UserSkills::find()
            .filter(crate::db::schema::user_skill::Column::UserId.eq(user_id))
            .all(&state.db)
            .await?;
        for s in existing_skills {
            let m: crate::db::schema::user_skill::ActiveModel = s.into();
            m.delete(&state.db).await?;
        }
    }

    // 导入 Todos
    for todo in &yaml_data.todos {
        // 检查是否已存在（按标题判断）
        let existing = UserTodos::find()
            .filter(crate::db::schema::user_todo::Column::UserId.eq(user_id))
            .filter(crate::db::schema::user_todo::Column::Title.eq(&todo.title))
            .one(&state.db)
            .await?;

        if existing.is_some() {
            if req.mode == "replace" {
                // replace 模式下先删再插
                if let Some(e) = existing {
                    let m: crate::db::schema::user_todo::ActiveModel = e.into();
                    m.delete(&state.db).await?;
                }
            } else {
                todos_skipped += 1;
                continue;
            }
        }

        let tag_names_json = serde_json::to_string(&todo.tag_names).unwrap_or_default();
        let sched_enabled = todo.scheduler_enabled.unwrap_or(false);

        let model = crate::db::schema::user_todo::ActiveModel {
            user_id: Set(user_id),
            title: Set(todo.title.clone()),
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

        model.insert(&state.db).await?;
        todos_imported += 1;
    }

    // 导入 Tags
    for tag in &yaml_data.tags {
        let existing = crate::db::schema::UserTags::find()
            .filter(crate::db::schema::user_tag::Column::UserId.eq(user_id))
            .filter(crate::db::schema::user_tag::Column::Name.eq(tag))
            .one(&state.db)
            .await?;

        if existing.is_none() {
            let model = crate::db::schema::user_tag::ActiveModel {
                user_id: Set(user_id),
                name: Set(tag.clone()),
                created_at: Set(now),
                ..Default::default()
            };
            model.insert(&state.db).await?;
            tags_imported += 1;
        }
    }

    // 导入 Skills
    for skill in &yaml_data.skills {
        let existing = crate::db::schema::UserSkills::find()
            .filter(crate::db::schema::user_skill::Column::UserId.eq(user_id))
            .filter(crate::db::schema::user_skill::Column::Name.eq(skill))
            .one(&state.db)
            .await?;

        if existing.is_none() {
            let model = crate::db::schema::user_skill::ActiveModel {
                user_id: Set(user_id),
                name: Set(skill.clone()),
                created_at: Set(now),
                ..Default::default()
            };
            model.insert(&state.db).await?;
            skills_imported += 1;
        }
    }

    tracing::info!(
        "导入完成: user_id={}, todos={}/{}, tags={}, skills={}, skipped={}",
        user_id, todos_imported, yaml_data.todos.len(), tags_imported, skills_imported, todos_skipped
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": ImportResult {
            todos_imported,
            tags_imported,
            skills_imported,
            todos_skipped,
        }
    })))
}
