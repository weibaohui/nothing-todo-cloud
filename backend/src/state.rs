//! 应用状态模块
//! 共享应用状态

use crate::config::Config;
use crate::db::DbConn;

/// 应用状态
pub struct AppState {
    pub db: DbConn,
    pub config: Config,
}

impl AppState {
    pub fn new(db: DbConn, config: Config) -> Self {
        Self { db, config }
    }
}
