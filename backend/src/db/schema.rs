//! 数据库 Schema 定义
//! 使用 SeaORM 定义所有数据表结构

use sea_orm::entity::prelude::*;

/// 用户表
#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique, length = 255)]
    pub email: String,
    /// bcrypt 哈希后的密码
    pub password_hash: String,
    pub created_at: DateTimeUtc,
    /// free / pro
    pub plan: String,
}

impl ActiveModelBehavior for ActiveModel {}

/// 设备表
#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "devices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column(name = "user_id"))]
    pub user_id: i64,
    /// 设备名称，如 "MacBook Pro"
    pub device_name: String,
    /// 设备公钥（可选，用于未来设备认证）
    pub device_key: Option<String>,
    pub last_seen_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
}

/// 设备数据快照表
#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "device_snapshots")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column(name = "device_id"))]
    pub device_id: i64,
    /// 递增版本号
    pub version: i64,
    /// 数据类型：todos / tags / skills / all
    pub data_type: String,
    /// 压缩后的数据（gzip + base64）
    pub data_payload: String,
    /// SHA256 校验和
    pub checksum: String,
    pub created_at: DateTimeUtc,
    /// 额外元数据（JSON）
    pub metadata: Option<String>,
}

/// 同步日志表
#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "sync_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column(name = "device_id"))]
    pub device_id: i64,
    /// 操作类型：push / pull / merge
    pub action: String,
    /// 状态：success / failed
    pub status: String,
    pub details: Option<String>,
    pub created_at: DateTimeUtc,
}

/// API Token 表
#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column(name = "user_id"))]
    pub user_id: i64,
    /// Token 名称，如 "Home Server"
    pub name: String,
    /// Token 哈希（不存储明文）
    pub token_hash: String,
    pub last_used_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
}
