//! 管理后台处理模块
//! 管理员查看用户、统计等

use crate::error::Result;
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total_users: i64,
    pub total_devices: i64,
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

/// 获取系统统计
pub async fn stats() -> Result<Json<UserStats>> {
    // TODO: 从数据库统计
    Ok(Json(UserStats {
        total_users: 1,
        total_devices: 4,
        total_syncs: 100,
    }))
}

/// 获取用户列表
pub async fn list_users() -> Result<Json<Vec<UserInfo>>> {
    // TODO: 从数据库查询
    Ok(Json(vec![UserInfo {
        id: 1,
        email: "admin@example.com".to_string(),
        plan: "pro".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }]))
}
