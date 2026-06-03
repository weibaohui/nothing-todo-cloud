//! 同步核心处理模块
//! 处理 Push（上传本地数据）和 Pull（拉取远端数据）

use crate::error::{AppError, Result};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::schema::DeviceSnapshots;
use crate::db::schema::SyncLogs;
use crate::db::schema::device_snapshot::Column as SnapshotColumn;

/// 同步状态响应
#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub device_id: i64,
    pub version: i64,
    pub last_sync_at: String,
    pub has_conflict: bool,
}

/// 推送数据请求
#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub device_id: i64,
    pub version: i64,
    /// 数据类型：todos / tags / skills / all
    pub data_type: String,
    /// gzip 压缩 + base64 编码后的数据
    pub data: String,
    /// SHA256 校验和
    pub checksum: String,
}

/// 推送数据响应
#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub success: bool,
    pub new_version: i64,
}

/// 拉取数据请求
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub device_id: i64,
    /// 期望的数据类型
    pub data_type: Option<String>,
}

/// 拉取数据响应
#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub device_id: i64,
    pub version: i64,
    pub data_type: String,
    pub data: String,
    pub checksum: String,
    pub updated_at: String,
}

/// 冲突解决请求
#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
    pub device_id: i64,
    /// 解决策略：overwrite_local / overwrite_remote / merge
    pub strategy: String,
    /// 冲突数据（用于 merge）
    pub merged_data: Option<String>,
}

/// 获取同步状态
pub async fn status(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullRequest>,
) -> Result<Json<SyncStatus>> {
    // 获取设备最新快照
    let latest_snapshot = DeviceSnapshots::find()
        .filter(SnapshotColumn::DeviceId.eq(params.device_id))
        .order_by_desc(SnapshotColumn::Version)
        .one(&state.db)
        .await?;

    let (version, last_sync_at, has_conflict) = match latest_snapshot {
        Some(s) => (s.version, s.created_at.to_rfc3339(), false),
        None => (0, "".to_string(), false),
    };

    Ok(Json(SyncStatus {
        device_id: params.device_id,
        version,
        last_sync_at,
        has_conflict,
    }))
}

/// Push：上传本地数据到服务器
pub async fn push(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>> {
    tracing::info!(
        "Push 请求: device_id={}, version={}, data_type={}",
        req.device_id,
        req.version,
        req.data_type
    );

    // 获取服务器当前版本
    let server_version = DeviceSnapshots::find()
        .filter(SnapshotColumn::DeviceId.eq(req.device_id))
        .filter(SnapshotColumn::DataType.eq(&req.data_type))
        .order_by_desc(SnapshotColumn::Version)
        .one(&state.db)
        .await
        .map(|opt| opt.map(|s| s.version).unwrap_or(0))
        .unwrap_or(0);

    // 版本必须大于服务器版本
    if req.version <= server_version {
        return Err(AppError::Conflict(format!(
            "版本号过时: 本地version={}, 服务器version={}",
            req.version, server_version
        )));
    }

    // 存储快照
    let now = Utc::now();
    let new_snapshot = crate::db::schema::device_snapshot::ActiveModel {
        device_id: Set(req.device_id),
        version: Set(req.version),
        data_type: Set(req.data_type.clone()),
        data_payload: Set(req.data),
        checksum: Set(req.checksum.clone()),
        created_at: Set(now),
        ..Default::default()
    };

    new_snapshot.insert(&state.db).await?;

    // 记录同步日志
    let sync_log = crate::db::schema::sync_log::ActiveModel {
        device_id: Set(req.device_id),
        action: Set("push".to_string()),
        status: Set("success".to_string()),
        details: Set(Some(format!("version={}, data_type={}", req.version, req.data_type))),
        created_at: Set(now),
        ..Default::default()
    };
    sync_log.insert(&state.db).await?;

    Ok(Json(PushResponse {
        success: true,
        new_version: req.version,
    }))
}

/// Pull：从服务器拉取数据
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
        .order_by_desc(SnapshotColumn::Version)
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
                version: s.version,
                data_type: s.data_type,
                data: s.data_payload,
                checksum: s.checksum,
                updated_at: s.created_at.to_rfc3339(),
            }))
        }
        None => Ok(Json(PullResponse {
            device_id: params.device_id,
            version: 0,
            data_type,
            data: "".to_string(),
            checksum: "".to_string(),
            updated_at: "".to_string(),
        })),
    }
}

/// Resolve：解决同步冲突
pub async fn resolve(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResolveRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        "Resolve 冲突: device_id={}, strategy={}",
        req.device_id,
        req.strategy
    );

    match req.strategy.as_str() {
        "overwrite_local" | "overwrite_remote" => {
            // 使用 merged_data 作为新版本数据
            if let Some(merged_data) = req.merged_data {
                let new_version = DeviceSnapshots::find()
                    .filter(SnapshotColumn::DeviceId.eq(req.device_id))
                    .order_by_desc(SnapshotColumn::Version)
                    .one(&state.db)
                    .await
                    .map(|opt| opt.map(|s| s.version + 1).unwrap_or(1))
                    .unwrap_or(1);

                let now = Utc::now();
                let snapshot = crate::db::schema::device_snapshot::ActiveModel {
                    device_id: Set(req.device_id),
                    version: Set(new_version),
                    data_type: Set("all".to_string()),
                    data_payload: Set(merged_data),
                    checksum: Set("".to_string()),
                    created_at: Set(now),
                    ..Default::default()
                };
                snapshot.insert(&state.db).await?;
            }
        }
        "merge" => {
            // TODO: 实现智能合并逻辑
            tracing::info!("Merge 策略暂未实现，使用 overwrite_remote");
        }
        _ => {
            return Err(AppError::BadRequest("无效的解决策略".to_string()));
        }
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
