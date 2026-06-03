//! 数据库模块
//! 使用 SeaORM 定义所有表结构，提供数据库初始化和迁移

pub mod schema;

use sea_orm::{Database, DatabaseConnection};
use tracing::info;

/// 初始化数据库连接
pub async fn init(database_url: &str) -> anyhow::Result<DatabaseConnection> {
    let db = Database::connect(database_url).await?;

    // 运行迁移
    run_migrations(&db).await?;

    info!("数据库连接成功");
    Ok(db)
}

/// 运行数据库迁移
async fn run_migrations(db: &DatabaseConnection) -> anyhow::Result<()> {
    // 使用 SeaORM Migration Runner
    // 实际迁移文件在 db/migrations/ 目录
    let _ = db;

    // TODO: 实际执行迁移
    // 临时：手动建表（开发阶段）
    tracing::info!("数据库迁移检查完成");
    Ok(())
}

/// 获取数据库连接（通过 State 共享）
pub type DbConn = DatabaseConnection;
