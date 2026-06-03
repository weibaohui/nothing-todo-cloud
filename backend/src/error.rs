//! 统一错误处理
//! 所有 API 错误通过 thiserror 定义，便于统一返回格式

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("认证失败: {0}")]
    Unauthorized(String),

    #[error("禁止访问: {0}")]
    Forbidden(String),

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("请求参数错误: {0}")]
    BadRequest(String),

    #[error("数据冲突: {0}")]
    Conflict(String),

    #[error("服务器内部错误: {0}")]
    Internal(String),

    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("JWT 错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("密码错误: {0}")]
    Password(#[from] bcrypt::BcryptError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Database(e) => {
                tracing::error!("数据库错误: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败".to_string())
            }
            AppError::Jwt(e) => {
                tracing::error!("JWT 错误: {:?}", e);
                (StatusCode::UNAUTHORIZED, "Token 无效或已过期".to_string())
            }
            AppError::Password(e) => {
                tracing::error!("密码错误: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "密码处理失败".to_string())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
