//! Token 管理处理模块
//! 处理 API Token 的创建、列表、撤销

use crate::error::Result;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub id: i64,
    pub name: String,
    pub token: Option<String>, // 仅创建时返回完整 token
    pub last_used_at: Option<String>,
    pub created_at: String,
}

/// 列出所有 Token
pub async fn list() -> Result<Json<Vec<TokenResponse>>> {
    // TODO: 从数据库查询用户的 Token 列表
    Ok(Json(vec![]))
}

/// 创建新 Token
pub async fn create(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<TokenResponse>> {
    let name = req["name"].as_str().unwrap_or("Unnamed Token");

    // TODO: 生成随机 Token
    // TODO: 哈希存储
    // TODO: 返回明文 Token（仅此时可见）

    tracing::info!("创建新 Token: {}", name);

    Ok(Json(TokenResponse {
        id: 1,
        name: name.to_string(),
        token: Some("ntd_cloud_xxx".to_string()),
        last_used_at: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }))
}

/// 撤销 Token
pub async fn revoke(Path(id): Path<i64>) -> Result<Json<serde_json::Value>> {
    // TODO: 从数据库删除 Token
    let _ = id;
    tracing::info!("撤销 Token: id={}", id);
    Ok(Json(serde_json::json!({ "success": true })))
}
