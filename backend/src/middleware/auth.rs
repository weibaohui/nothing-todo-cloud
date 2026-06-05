//! JWT 认证中间件
//! 支持两种 Token：
//! 1. JWT Token - 登录 Token，用于创建 API Token
//! 2. API Token - ntd_xxx 格式，存储在 api_tokens 表中

use crate::error::AppError;
use crate::services::auth_service::Claims;
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;

/// 从请求中提取并验证 Token
/// 支持 JWT Token 和 API Token
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("缺少认证 Token".to_string()))?;

    let claims = verify_token(token, &state).await
        .map_err(|_| AppError::Unauthorized("Token 无效或已过期".to_string()))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// 验证 Token
/// 1. 先尝试 JWT 验证
/// 2. 如果是 ntd_xxx 格式，查询 api_tokens 表验证
async fn verify_token(token: &str, state: &Arc<AppState>) -> Result<Claims, AppError> {
    // 1. 尝试 JWT 验证
    if let Ok(claims) = crate::services::auth_service::verify_token(token, &state.config.jwt.secret) {
        return Ok(claims);
    }

    // 2. 尝试 API Token 验证 (ntd_xxx 格式)
    if token.starts_with("ntd_") {
        return verify_api_token(token, state).await;
    }

    Err(AppError::Unauthorized("Token 无效".to_string()))
}

/// 验证 API Token（通过哈希比对）
async fn verify_api_token(token: &str, state: &Arc<AppState>) -> Result<Claims, AppError> {
    use crate::db::schema::ApiTokens;
    use crate::db::schema::api_token::Column as TokenColumn;

    // 对传入的 token 进行哈希，然后比对存储的哈希值
    let token_hash = hash_token(token);

    let api_token = ApiTokens::find()
        .filter(TokenColumn::TokenHash.eq(&token_hash))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Token 无效".to_string()))?;

    Ok(Claims {
        sub: api_token.user_id,
        device_id: Some(api_token.id),
        token_type: "api".to_string(),
        exp: 0,
    })
}

/// 对 Token 进行哈希
fn hash_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
