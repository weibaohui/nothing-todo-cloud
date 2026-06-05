//! 数据库模块
//! 使用 SeaORM 定义所有表结构，提供数据库初始化和迁移

pub mod schema;

use sea_orm::{Database, DatabaseConnection, DatabaseBackend, Statement, ConnectionTrait};
use tracing::info;

/// 初始化数据库连接
pub async fn init(database_url: &str) -> anyhow::Result<DatabaseConnection> {
    let db = Database::connect(database_url).await?;

    // 运行迁移
    run_migrations(&db).await?;

    info!("数据库连接成功");
    Ok(db)
}

/// 运行数据库迁移 - 创建表结构
async fn run_migrations(db: &DatabaseConnection) -> anyhow::Result<()> {
    // 创建 users 表
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            plan TEXT NOT NULL DEFAULT 'free',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    ))
    .await?;

    // 创建 devices 表
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS devices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            device_name TEXT NOT NULL,
            device_key TEXT,
            last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    ))
    .await?;

    // 创建 api_tokens 表
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS api_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            token TEXT NOT NULL,
            last_used_at TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    ))
    .await?;

    // 迁移：检查是否有旧的 token_hash 列，如有则迁移数据并删除
    let has_hash_column = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='api_tokens' AND sql LIKE '%token_hash%'",
        ))
        .await
        .is_ok();

    if has_hash_column {
        // 添加新列 token（如果不存在）
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "ALTER TABLE api_tokens ADD COLUMN token TEXT",
        ))
        .await
        .ok();

        // 复制 token_hash 的值到 token 列（格式化为 ntd_id 作为占位符）
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "UPDATE api_tokens SET token = 'ntd_' || id WHERE token IS NULL OR token = ''",
        ))
        .await
        .ok();

        tracing::info!("已迁移 api_tokens 表：token_hash -> token");
    }

    // 创建 user_todos 表
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS user_todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            prompt TEXT,
            status TEXT DEFAULT 'pending',
            executor TEXT,
            scheduler_enabled INTEGER DEFAULT 0,
            scheduler_config TEXT,
            tag_names TEXT,
            workspace TEXT,
            worktree TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT,
            UNIQUE(user_id, title)
        )
        "#,
    ))
    .await?;

    // 创建 user_tags 表
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS user_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, name)
        )
        "#,
    ))
    .await?;

    // 创建 user_skills 表
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS user_skills (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, name)
        )
        "#,
    ))
    .await?;

    // 创建 sync_logs 表
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS sync_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            status TEXT NOT NULL,
            details TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    ))
    .await?;

    tracing::info!("数据库迁移完成");
    Ok(())
}

/// 获取数据库连接（通过 State 共享）
pub type DbConn = DatabaseConnection;
