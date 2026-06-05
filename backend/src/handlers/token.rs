//! Token 管理处理模块
//! 处理 API Token 的创建、列表、撤销
//! 安全设计：Token 存储哈希值，列表接口不返回明文

use crate::error::{AppError, Result};
use crate::services::auth_service::Claims;
use crate::state::AppState;
use axum::{
    extract::{Path, State, Extension},
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::schema::ApiTokens;
use crate::db::schema::api_token::Column as TokenColumn;

/// 对 Token 进行哈希（不可逆存储）
fn hash_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub id: i64,
    pub name: String,
    pub token: Option<String>, // 仅创建时返回完整 token（一次性的）
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
}

/// 列出所有 Token
/// 安全：列表不返回明文 token
pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<TokenResponse>>> {
    let tokens = ApiTokens::find()
        .filter(TokenColumn::UserId.eq(claims.sub))
        .all(&state.db)
        .await?;

    // 安全：列表不返回明文 token
    let response: Vec<TokenResponse> = tokens
        .into_iter()
        .map(|t| TokenResponse {
            id: t.id,
            name: t.name,
            token: None, // 不返回 token 明文
            last_used_at: t.last_used_at.map(|dt| dt.to_rfc3339()),
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

/// 创建新 Token
/// 安全：存储哈希值，只返回一次明文
pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateTokenRequest>,
) -> Result<Json<TokenResponse>> {
    // 生成随机 Token
    let raw_token = format!("ntd_{}", Uuid::new_v4());
    let token_hash = hash_token(&raw_token);

    tracing::info!("创建新 Token: {} for user {}", req.name, claims.sub);

    let now = Utc::now();
    let new_token = crate::db::schema::api_token::ActiveModel {
        user_id: Set(claims.sub),
        name: Set(req.name.clone()),
        token_hash: Set(token_hash), // 只存储哈希
        created_at: Set(now),
        ..Default::default()
    };

    let token = new_token.insert(&state.db).await?;

    Ok(Json(TokenResponse {
        id: token.id,
        name: token.name,
        token: Some(raw_token), // 创建时返回明文 token（一次性的，用户需立即保存）
        last_used_at: None,
        created_at: token.created_at.to_rfc3339(),
    }))
}

/// 撤销 Token
pub async fn revoke(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>> {
    let token = ApiTokens::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Token 不存在".to_string()))?;

    // 验证 Token 属于当前用户
    if token.user_id != claims.sub {
        return Err(AppError::Forbidden("无权撤销此 Token".to_string()));
    }

    let active_model: crate::db::schema::api_token::ActiveModel = token.into();
    active_model.delete(&state.db).await?;

    tracing::info!("撤销 Token: id={}", id);
    Ok(Json(serde_json::json!({ "success": true })))
}

/// 验证 API Token（通过哈希比对）
pub async fn verify_token(token: &str, state: &Arc<AppState>) -> Result<i64> {
    let token_hash = hash_token(token);

    let api_token = ApiTokens::find()
        .filter(TokenColumn::TokenHash.eq(&token_hash))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Token 无效".to_string()))?;

    Ok(api_token.user_id)
}
