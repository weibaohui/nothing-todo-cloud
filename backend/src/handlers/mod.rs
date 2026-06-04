//! API Handlers 模块
//! 各个端点的处理函数

pub mod admin;
pub mod auth;
pub mod device;
pub mod sync;
pub mod token;

use axum::Json;
use serde_json::json;

/// 健康检查（包含服务状态）
pub async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "nothing-todo-cloud"
    }))
}

/// 存活探针（仅验证服务可达）
/// 用于 Kubernetes livenessProbe 或负载均衡健康检查
/// 返回 200 即表示存活，不涉及业务逻辑检查
pub async fn livez() -> &'static str {
    "OK"
}
