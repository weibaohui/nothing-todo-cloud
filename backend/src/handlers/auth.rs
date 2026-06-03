//! 认证处理模块
//! 处理用户注册、登录、登出

use crate::error::{AppError, Result};
use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 应用状态
pub type AppState = Arc<()>;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user_id: Option<i64>,
}

/// 用户注册
pub async fn register(
    State(_state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    // 验证邮箱格式
    if !req.email.contains('@') {
        return Err(AppError::BadRequest("无效的邮箱格式".to_string()));
    }

    // 验证密码强度
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("密码长度至少 6 位".to_string()));
    }

    // TODO: 密码哈希 + 存储到数据库
    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;

    tracing::info!("用户注册: {}", req.email);

    // TODO: 插入数据库

    Ok(Json(AuthResponse {
        success: true,
        token: Some("jwt-token-placeholder".to_string()),
        user_id: Some(1),
    }))
}

/// 用户登录
pub async fn login(
    State(_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    // TODO: 验证邮箱密码
    let _password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;

    tracing::info!("用户登录: {}", req.email);

    // TODO: 生成 JWT Token

    Ok(Json(AuthResponse {
        success: true,
        token: Some("jwt-token-placeholder".to_string()),
        user_id: Some(1),
    }))
}

/// 用户登出（前端清除 Token 即可，服务端可选支持黑名单）
pub async fn logout() -> Json<AuthResponse> {
    Json(AuthResponse {
        success: true,
        token: None,
        user_id: None,
    })
}
