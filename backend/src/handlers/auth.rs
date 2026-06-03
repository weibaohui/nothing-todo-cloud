//! 认证处理模块
//! 处理用户注册、登录、登出

use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::services::auth_service::{generate_token, hash_password, verify_password};
use axum::{
    extract::State,
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::schema::Users;

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
    State(state): State<Arc<AppState>>,
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

    // 密码哈希
    let password_hash = hash_password(&req.password)?;

    tracing::info!("用户注册: {}", req.email);

    // 检查邮箱是否已存在
    let existing = Users::find()
        .filter(crate::db::schema::user::Column::Email.eq(&req.email))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("该邮箱已注册".to_string()));
    }

    // 创建用户
    let new_user = crate::db::schema::user::ActiveModel {
        email: Set(req.email.clone()),
        password_hash: Set(password_hash),
        plan: Set("free".to_string()),
        ..Default::default()
    };

    let user = new_user.insert(&state.db).await?;

    // 生成 JWT Token
    let token = generate_token(
        user.id,
        None,
        &state.config.jwt.secret,
        state.config.jwt.expiration_hours,
    )?;

    Ok(Json(AuthResponse {
        success: true,
        token: Some(token),
        user_id: Some(user.id),
    }))
}

/// 用户登录
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    tracing::info!("用户登录: {}", req.email);

    // 查找用户
    let user = Users::find()
        .filter(crate::db::schema::user::Column::Email.eq(&req.email))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("邮箱或密码错误".to_string()))?;

    // 验证密码
    if !verify_password(&req.password, &user.password_hash) {
        return Err(AppError::Unauthorized("邮箱或密码错误".to_string()));
    }

    // 生成 JWT Token
    let token = generate_token(
        user.id,
        None,
        &state.config.jwt.secret,
        state.config.jwt.expiration_hours,
    )?;

    Ok(Json(AuthResponse {
        success: true,
        token: Some(token),
        user_id: Some(user.id),
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
