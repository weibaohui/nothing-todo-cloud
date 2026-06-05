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

    // 创建 api_tokens 表（使用 token_hash 存储）
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS api_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            token_hash TEXT NOT NULL,
            last_used_at TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    ))
    .await?;

    // 迁移：如果存在旧的 token 明文字段，迁移到 token_hash
    // 检查 token 列是否存在
    let has_plain_token = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='api_tokens' AND sql LIKE '%token TEXT%' AND sql NOT LIKE '%token_hash%'",
        ))
        .await?;

    if let Some(_row) = has_plain_token {
        // 添加 token_hash 列
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "ALTER TABLE api_tokens ADD COLUMN token_hash TEXT",
        ))
        .await
        .ok();

        // 将 token 明文复制到 token_hash（这只是占位符，实际需要用户重新创建 Token）
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "UPDATE api_tokens SET token_hash = 'ntd_' || id WHERE token_hash IS NULL OR token_hash = ''",
        ))
        .await
        .ok();

        info!("已迁移 api_tokens 表：token 明文 -> token_hash（用户需重新创建 Token）");
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
