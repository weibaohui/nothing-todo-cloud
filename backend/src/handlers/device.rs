//! 设备管理处理模块
//! 处理设备注册、查询、更新、删除

use crate::error::{AppError, Result};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::schema::Devices;

/// 设备信息响应
#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: i64,
    pub device_name: String,
    pub last_seen_at: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_name: String,
    pub device_key: Option<String>,
}

/// 注册新设备
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<DeviceResponse>> {
    let now = Utc::now();

    let new_device = crate::db::schema::device::ActiveModel {
        user_id: Set(1), // TODO: 从 JWT 获取用户 ID
        device_name: Set(req.device_name),
        device_key: Set(req.device_key),
        last_seen_at: Set(now),
        created_at: Set(now),
        ..Default::default()
    };

    let device = new_device.insert(&state.db).await?;

    Ok(Json(DeviceResponse {
        id: device.id,
        device_name: device.device_name,
        last_seen_at: device.last_seen_at.to_rfc3339(),
        created_at: device.created_at.to_rfc3339(),
    }))
}

/// 列出用户的所有设备
pub async fn list(State(state): State<Arc<AppState>>) -> Result<Json<Vec<DeviceResponse>>> {
    let devices = Devices::find()
        .filter(crate::db::schema::device::Column::UserId.eq(1)) // TODO: 从 JWT 获取用户 ID
        .all(&state.db)
        .await?;

    let response: Vec<DeviceResponse> = devices
        .into_iter()
        .map(|d| DeviceResponse {
            id: d.id,
            device_name: d.device_name,
            last_seen_at: d.last_seen_at.to_rfc3339(),
            created_at: d.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

/// 获取设备详情
pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<DeviceResponse>> {
    let device = Devices::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_string()))?;

    Ok(Json(DeviceResponse {
        id: device.id,
        device_name: device.device_name,
        last_seen_at: device.last_seen_at.to_rfc3339(),
        created_at: device.created_at.to_rfc3339(),
    }))
}

/// 删除设备
pub async fn delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>> {
    let device = Devices::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_string()))?;

    let mut active_model: crate::db::schema::device::ActiveModel = device.into();
    active_model.delete(&state.db).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateNameRequest {
    pub name: String,
}

/// 更新设备名称
pub async fn update_name(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateNameRequest>,
) -> Result<Json<DeviceResponse>> {
    let device = Devices::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_string()))?;

    let mut active_model: crate::db::schema::device::ActiveModel = device.into();
    active_model.device_name = Set(req.name.clone());
    let updated = active_model.update(&state.db).await?;

    Ok(Json(DeviceResponse {
        id: updated.id,
        device_name: updated.device_name,
        last_seen_at: updated.last_seen_at.to_rfc3339(),
        created_at: updated.created_at.to_rfc3339(),
    }))
}
