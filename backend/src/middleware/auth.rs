//! JWT 认证中间件
//! 验证请求头中的 Bearer Token，提取 user_id 和 device_id

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

/// JWT Claims 结构
#[derive(Debug, serde::Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: i64,
    /// 设备 ID（可选）
    pub device_id: Option<i64>,
    /// Token 类型：access / refresh
    pub token_type: String,
    /// 过期时间
    pub exp: usize,
}

/// 从请求中提取并验证 JWT Token
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // 从 Authorization Header 提取 Token
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;

    // TODO: 验证 JWT
    // let claims = verify_jwt(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 将 Claims 注入到请求扩展中
    // req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// 可选认证（不强制要求 Token 存在）
pub async fn optional_auth_middleware(req: Request, next: Next) -> Response {
    // 如果有 Token 就验证，没有就继续
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        // TODO: 验证并注入 Claims
        let _ = token;
    }

    next.run(req).await
}
