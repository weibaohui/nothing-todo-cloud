//! 同步服务
//! 处理 Push/Pull 数据同步核心逻辑

use crate::db::DbConn;

/// 同步状态
#[derive(Debug, serde::Serialize)]
pub struct SyncStatus {
    pub device_id: i64,
    pub version: i64,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub has_conflict: bool,
}

/// 数据快照
#[derive(Debug, serde::Serialize)]
pub struct DataSnapshot {
    pub id: i64,
    pub device_id: i64,
    pub version: i64,
    pub data_type: String,
    pub data_payload: String,
    pub checksum: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Push：存储设备上传的数据
pub async fn push_data(
    db: &DbConn,
    device_id: i64,
    version: i64,
    data_type: &str,
    data: &[u8],
    checksum: &str,
) -> anyhow::Result<i64> {
    let _ = (db, device_id, version, data_type, data, checksum);
    // TODO: 实现数据存储逻辑
    Ok(version + 1)
}

/// Pull：获取设备最新数据
pub async fn pull_data(
    db: &DbConn,
    device_id: i64,
    data_type: Option<&str>,
) -> anyhow::Result<Option<DataSnapshot>> {
    let _ = (db, device_id, data_type);
    // TODO: 实现数据拉取逻辑
    Ok(None)
}

/// 检测冲突
pub async fn detect_conflict(
    db: &DbConn,
    device_id: i64,
    local_version: i64,
) -> anyhow::Result<bool> {
    let _ = (db, device_id, local_version);
    // TODO: 检测服务器是否有更新
    Ok(false)
}

/// 获取设备当前版本
pub async fn get_device_version(
    db: &DbConn,
    device_id: i64,
) -> anyhow::Result<i64> {
    let _ = (db, device_id);
    // TODO: 从数据库获取当前版本
    Ok(0)
}
