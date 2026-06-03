//! 设备管理处理模块
//! 处理设备注册、查询、更新、删除

use crate::error::Result;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

/// 设备信息响应
#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: i64,
    pub device_name: String,
    pub last_seen_at: String,
    pub created_at: String,
}

/// 注册新设备
pub async fn register() -> Result<Json<DeviceResponse>> {
    // TODO: 实现设备注册逻辑
    Ok(Json(DeviceResponse {
        id: 1,
        device_name: "New Device".to_string(),
        last_seen_at: "2026-01-01T00:00:00Z".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }))
}

/// 列出用户的所有设备
pub async fn list() -> Result<Json<Vec<DeviceResponse>>> {
    // TODO: 从数据库查询用户设备列表
    Ok(Json(vec![]))
}

/// 获取设备详情
pub async fn get(Path(id): Path<i64>) -> Result<Json<DeviceResponse>> {
    // TODO: 从数据库查询设备详情
    let _ = id;
    Ok(Json(DeviceResponse {
        id,
        device_name: "Device".to_string(),
        last_seen_at: "2026-01-01T00:00:00Z".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }))
}

/// 删除设备
pub async fn delete(Path(id): Path<i64>) -> Result<Json<serde_json::Value>> {
    // TODO: 从数据库删除设备
    let _ = id;
    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateNameRequest {
    pub name: String,
}

/// 更新设备名称
pub async fn update_name(
    Path(id): Path<i64>,
    Json(req): Json<UpdateNameRequest>,
) -> Result<Json<DeviceResponse>> {
    let _ = id;
    let _ = req.name;
    // TODO: 更新设备名称
    Ok(Json(DeviceResponse {
        id,
        device_name: "Updated Name".to_string(),
        last_seen_at: "2026-01-01T00:00:00Z".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }))
}
