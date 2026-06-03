//! JWT 认证中间件
//! 验证请求头中的 Bearer Token，提取 user_id 和 device_id

use crate::error::AppError;
use crate::services::auth_service::Claims;
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// 从请求中提取并验证 JWT Token
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 从 Authorization Header 提取 Token
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("缺少认证 Token".to_string()))?;

    // 验证 JWT
    let claims = crate::services::auth_service::verify_token(token, &state.config.jwt.secret)
        .map_err(|_| AppError::Unauthorized("Token 无效或已过期".to_string()))?;

    // 将 Claims 注入到请求扩展中
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// 可选认证（不强制要求 Token 存在）
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    // 如果有 Token 就验证，没有就继续
    let claims = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            crate::services::auth_service::verify_token(token, &state.config.jwt.secret).ok()
        });

    if let Some(claims) = claims {
        req.extensions_mut().insert(claims);
    }

    next.run(req).await
}
