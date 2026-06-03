//! 设备服务
//! 处理设备注册、更新、删除

use crate::db::DbConn;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

/// 设备信息
#[derive(Debug, serde::Serialize)]
pub struct DeviceInfo {
    pub id: i64,
    pub user_id: i64,
    pub device_name: String,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 设备注册
pub async fn register_device(
    db: &DbConn,
    user_id: i64,
    device_name: String,
) -> anyhow::Result<DeviceInfo> {
    // TODO: 实现设备注册逻辑
    let _ = (db, user_id, device_name);

    Ok(DeviceInfo {
        id: 1,
        user_id,
        device_name: "New Device".to_string(),
        last_seen_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    })
}

/// 更新设备最后访问时间
pub async fn update_last_seen(
    db: &DbConn,
    device_id: i64,
) -> anyhow::Result<()> {
    let _ = (db, device_id);
    // TODO: 更新 last_seen_at
    Ok(())
}
