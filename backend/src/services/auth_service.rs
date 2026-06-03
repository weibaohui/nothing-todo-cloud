//! 认证服务
//! 处理用户注册、登录、Token 生成与验证

use crate::db::DbConn;

/// 生成 JWT Token
pub fn generate_token(
    user_id: i64,
    device_id: Option<i64>,
    secret: &str,
    expiration_hours: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::{Duration, Utc};

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expiration_hours as i64))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = crate::middleware::auth::Claims {
        sub: user_id,
        device_id,
        token_type: "access".to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// 验证 JWT Token
pub fn verify_token(
    token: &str,
    secret: &str,
) -> Result<crate::middleware::auth::Claims, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    let token_data = decode::<crate::middleware::auth::Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// 密码哈希
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

/// 验证密码
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}
