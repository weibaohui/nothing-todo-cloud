//! nothing-todo-cloud 主入口
//! 提供设备注册、Token 认证、数据同步功能

mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod services;
mod state;

use axum::{
    routing::{get, post, delete, put},
    Router,
    response::IntoResponse,
};
use axum::middleware as axum_middleware;
use rust_embed::Embed;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::AppState;
use crate::middleware::auth::auth_middleware;

/// 前端静态文件嵌入
#[derive(Embed)]
#[folder = "../frontend/dist"]
struct Assets;

/// 获取嵌入的前端文件
fn get_embedded_file(path: &str) -> Option<Vec<u8>> {
    // 尝试直接获取文件
    let path = path.trim_start_matches('/');
    if let Some(file) = Assets::get(path) {
        return Some(file.data.to_vec());
    }
    // SPA 路由：返回 index.html
    Assets::get("index.html").map(|f| f.data.to_vec())
}

/// 获取文件 MIME 类型
fn get_mime_type(path: &str) -> &'static str {
    let path = path.trim_start_matches('/');
    if path.is_empty() || path == "index.html" {
        "text/html; charset=utf-8"
    } else if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else {
        "application/octet-stream"
    }
}

/// 前端静态文件服务（支持 SPA）
async fn serve_static(path: String) -> impl IntoResponse {
    let mime = get_mime_type(&path);
    if let Some(data) = get_embedded_file(&path) {
        ([(axum::http::header::CONTENT_TYPE, mime)], data)
    } else {
        // 文件不存在，返回 index.html（SPA fallback）
        if let Some(data) = Assets::get("index.html") {
            ([(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")], data.data.to_vec())
        } else {
            ([(axum::http::header::CONTENT_TYPE, "text/plain")], b"Frontend not built".to_vec())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("启动 nothing-todo-cloud 服务器...");

    // 加载配置
    let config = config::Config::load()?;

    // 初始化数据库
    let db = db::init(&config.database.url).await?;

    tracing::info!("数据库初始化完成");

    // 创建应用状态
    let state = Arc::new(state::AppState::new(db, config.clone()));

    // 构建路由
    let app = build_app(state.clone());

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("服务器监听在 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_app(state: Arc<AppState>) -> Router {
    // CORS 配置
    let cors = CorsLayer::permissive();

    // 公开路由（无需认证）
    let public_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/livez", get(handlers::livez))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/logout", post(handlers::auth::logout));

    // 受保护的路由（需要认证）
    let protected_routes = Router::new()
        .route("/api/tokens", get(handlers::token::list))
        .route("/api/tokens", post(handlers::token::create))
        .route("/api/tokens/:id", delete(handlers::token::revoke))
        .route("/api/devices", get(handlers::device::list))
        .route("/api/devices", post(handlers::device::register))
        .route("/api/devices/:id", get(handlers::device::get))
        .route("/api/devices/:id", delete(handlers::device::delete))
        .route("/api/devices/:id", put(handlers::device::update_name))
        .route("/api/v1/sync/status", get(handlers::sync::status))
        .route("/api/v1/sync/push", post(handlers::sync::push))
        .route("/api/v1/sync/pull", get(handlers::sync::pull))
        .route("/api/admin/users", get(handlers::admin::list_users))
        .route("/api/admin/stats", get(handlers::admin::stats))
        .layer(axum_middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // 前端静态文件（SPA fallback）
        .route("/*path", get(serve_static))
        .route("/", get(serve_static))
        .with_state(state)
        .layer(cors)
}
