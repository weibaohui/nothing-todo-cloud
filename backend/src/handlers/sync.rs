//! 同步核心处理模块
//! 采用全量覆盖策略：Push 直接存储，Pull 直接返回

use crate::error::Result;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::schema::DeviceSnapshots;
use crate::db::schema::device_snapshot::Column as SnapshotColumn;

/// 同步状态响应
#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub device_id: i64,
    pub last_sync_at: String,
}

/// 推送数据请求（全量覆盖）
#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub device_id: i64,
    /// 数据类型：todos / tags / skills / all
    pub data_type: String,
    /// gzip 压缩 + base64 编码后的数据
    pub data: String,
    /// SHA256 校验和（可选）
    pub checksum: Option<String>,
}

/// 推送数据响应
#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub success: bool,
}

/// 拉取数据请求
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub device_id: i64,
    /// 期望的数据类型（可选，默认 all）
    pub data_type: Option<String>,
}

/// 拉取数据响应
#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub device_id: i64,
    pub data_type: String,
    pub data: String,
    pub checksum: String,
    pub updated_at: String,
}

/// 获取同步状态
pub async fn status(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullRequest>,
) -> Result<Json<SyncStatus>> {
    let data_type = params.data_type.clone().unwrap_or_else(|| "all".to_string());

    // 获取设备最新快照
    let latest_snapshot = DeviceSnapshots::find()
        .filter(SnapshotColumn::DeviceId.eq(params.device_id))
        .filter(SnapshotColumn::DataType.eq(&data_type))
        .one(&state.db)
        .await?;

    let last_sync_at = latest_snapshot
        .map(|s| s.created_at.to_rfc3339())
        .unwrap_or_default();

    Ok(Json(SyncStatus {
        device_id: params.device_id,
        last_sync_at,
    }))
}

/// Push：上传本地数据（全量覆盖服务器）
pub async fn push(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>> {
    tracing::info!(
        "Push 请求: device_id={}, data_type={}, data_len={}",
        req.device_id,
        req.data_type,
        req.data.len()
    );

    let now = Utc::now();

    // 删除该设备的旧快照（同一 data_type）
    let old_snapshots = DeviceSnapshots::find()
        .filter(SnapshotColumn::DeviceId.eq(req.device_id))
        .filter(SnapshotColumn::DataType.eq(&req.data_type))
        .all(&state.db)
        .await?;

    for old in old_snapshots {
        let mut model: crate::db::schema::device_snapshot::ActiveModel = old.into();
        model.delete(&state.db).await?;
    }

    // 存储新快照
    let new_snapshot = crate::db::schema::device_snapshot::ActiveModel {
        device_id: Set(req.device_id),
        version: Set(1),  // 不再使用版本号，简单设为1
        data_type: Set(req.data_type.clone()),
        data_payload: Set(req.data),
        checksum: Set(req.checksum.unwrap_or_default()),
        created_at: Set(now),
        ..Default::default()
    };

    new_snapshot.insert(&state.db).await?;

    // 记录同步日志
    let sync_log = crate::db::schema::sync_log::ActiveModel {
        device_id: Set(req.device_id),
        action: Set("push".to_string()),
        status: Set("success".to_string()),
        details: Set(Some(format!("data_type={}", req.data_type))),
        created_at: Set(now),
        ..Default::default()
    };
    sync_log.insert(&state.db).await?;

    Ok(Json(PushResponse { success: true }))
}

/// Pull：拉取服务器数据（全量返回）
pub async fn pull(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullRequest>,
) -> Result<Json<PullResponse>> {
    tracing::info!("Pull 请求: device_id={}", params.device_id);

    let data_type = params.data_type.clone().unwrap_or_else(|| "all".to_string());

    // 获取最新快照
    let snapshot = DeviceSnapshots::find()
        .filter(SnapshotColumn::DeviceId.eq(params.device_id))
        .filter(SnapshotColumn::DataType.eq(&data_type))
        .one(&state.db)
        .await?;

    match snapshot {
        Some(s) => {
            // 更新设备最后访问时间
            let now = Utc::now();
            if let Some(device) = crate::db::schema::Devices::find_by_id(params.device_id)
                .one(&state.db)
                .await?
            {
                let mut device: crate::db::schema::device::ActiveModel = device.into();
                device.last_seen_at = Set(now);
                device.update(&state.db).await?;
            }

            Ok(Json(PullResponse {
                device_id: params.device_id,
                data_type: s.data_type,
                data: s.data_payload,
                checksum: s.checksum,
                updated_at: s.created_at.to_rfc3339(),
            }))
        }
        None => Ok(Json(PullResponse {
            device_id: params.device_id,
            data_type,
            data: "".to_string(),
            checksum: "".to_string(),
            updated_at: "".to_string(),
        })),
    }
}
