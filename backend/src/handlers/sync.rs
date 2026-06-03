//! 同步核心处理模块
//! 处理 Push（上传本地数据）和 Pull（拉取远端数据）

use crate::error::Result;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

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
pub async fn status() -> Result<Json<SyncStatus>> {
    // TODO: 从数据库获取设备同步状态
    Ok(Json(SyncStatus {
        device_id: 1,
        version: 0,
        last_sync_at: "2026-01-01T00:00:00Z".to_string(),
        has_conflict: false,
    }))
}

/// Push：上传本地数据到服务器
pub async fn push(
    State(_state): State<()>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>> {
    tracing::info!(
        "Push 请求: device_id={}, version={}, data_type={}",
        req.device_id,
        req.version,
        req.data_type
    );

    // TODO: 验证 checksum
    // TODO: 检查版本号（version 必须大于服务器存储的版本）
    // TODO: 存储数据快照
    // TODO: 返回新版本号

    Ok(Json(PushResponse {
        success: true,
        new_version: req.version + 1,
    }))
}

/// Pull：从服务器拉取数据
pub async fn pull(
    Query(params): Query<PullRequest>,
) -> Result<Json<PullResponse>> {
    tracing::info!("Pull 请求: device_id={}", params.device_id);

    // TODO: 获取设备最新数据快照
    // TODO: 验证请求合法性

    Ok(Json(PullResponse {
        device_id: params.device_id,
        version: 1,
        data_type: params.data_type.unwrap_or_else(|| "all".to_string()),
        data: "".to_string(),
        checksum: "".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }))
}

/// Resolve：解决同步冲突
pub async fn resolve(
    Json(req): Json<ResolveRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        "Resolve 冲突: device_id={}, strategy={}",
        req.device_id,
        req.strategy
    );

    // TODO: 根据策略解决冲突
    // - overwrite_local: 用服务器数据覆盖本地
    // - overwrite_remote: 用本地数据覆盖服务器
    // - merge: 合并数据

    Ok(Json(serde_json::json!({ "success": true })))
}
