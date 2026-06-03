//! API Handlers 模块
//! 各个端点的处理函数

pub mod admin;
pub mod auth;
pub mod device;
pub mod sync;
pub mod token;

use axum::Json;
use serde_json::json;

/// 健康检查
pub async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "nothing-todo-cloud"
    }))
}
