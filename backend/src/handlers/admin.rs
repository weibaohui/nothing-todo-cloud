//! 管理后台处理模块
//! 管理员查看用户、统计、同步日志等

use crate::error::Result;
use crate::state::AppState;
use axum::{extract::State, Json};
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder, QuerySelect};
use serde::Serialize;
use std::sync::Arc;

use crate::db::schema::{Users, SyncLogs};

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total_users: i64,
    pub total_syncs: i64,
}

/// 用户列表响应
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
    pub plan: String,
    pub created_at: String,
}

/// 同步日志条目
#[derive(Debug, Serialize)]
pub struct SyncLogEntry {
    pub id: i64,
    pub user_id: i64,
    pub action: String,
    pub status: String,
    pub details: Option<String>,
    pub created_at: String,
}

/// 获取系统统计
pub async fn stats(State(state): State<Arc<AppState>>) -> Result<Json<UserStats>> {
    let total_users = Users::find().count(&state.db).await?;
    let total_syncs = SyncLogs::find().count(&state.db).await?;

    Ok(Json(UserStats {
        total_users: total_users as i64,
        total_syncs: total_syncs as i64,
    }))
}

/// 获取用户列表
pub async fn list_users(State(state): State<Arc<AppState>>) -> Result<Json<Vec<UserInfo>>> {
    let users_list = Users::find().all(&state.db).await?;

    let response: Vec<UserInfo> = users_list
        .into_iter()
        .map(|u| UserInfo {
            id: u.id,
            email: u.email,
            plan: u.plan,
            created_at: u.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

/// 获取同步日志列表（按时间倒序，最多 50 条）
pub async fn sync_logs(State(state): State<Arc<AppState>>) -> Result<Json<Vec<SyncLogEntry>>> {
    use crate::db::schema::sync_log;

    let logs = SyncLogs::find()
        .order_by_desc(sync_log::Column::CreatedAt)
        .limit(50)
        .all(&state.db)
        .await?;

    let response: Vec<SyncLogEntry> = logs
        .into_iter()
        .map(|l| SyncLogEntry {
            id: l.id,
            user_id: l.user_id,
            action: l.action,
            status: l.status,
            details: l.details,
            created_at: l.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}
