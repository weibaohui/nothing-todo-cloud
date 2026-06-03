//! Token 管理处理模块
//! 处理 API Token 的创建、列表、撤销

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

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub id: i64,
    pub name: String,
    pub token: Option<String>, // 仅创建时返回完整 token
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
}

/// 列出所有 Token
pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<TokenResponse>>> {
    let tokens = ApiTokens::find()
        .filter(TokenColumn::UserId.eq(claims.sub))
        .all(&state.db)
        .await?;

    let response: Vec<TokenResponse> = tokens
        .into_iter()
        .map(|t| TokenResponse {
            id: t.id,
            name: t.name,
            token: None, // 不返回 token 哈希
            last_used_at: t.last_used_at.map(|dt| dt.to_rfc3339()),
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

/// 创建新 Token
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
        token_hash: Set(token_hash),
        created_at: Set(now),
        ..Default::default()
    };

    let token = new_token.insert(&state.db).await?;

    Ok(Json(TokenResponse {
        id: token.id,
        name: token.name,
        token: Some(raw_token), // 仅创建时返回明文
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

/// 对 Token 进行哈希
fn hash_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
