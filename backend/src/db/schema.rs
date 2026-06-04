//! 数据库 Schema 定义
//! 使用 SeaORM 定义所有数据表结构

pub mod user;
pub mod device;
pub mod api_token;
pub mod user_snapshot;
pub mod sync_log;

pub use user::Entity as Users;
pub use device::Entity as Devices;
pub use api_token::Entity as ApiTokens;
pub use user_snapshot::Entity as UserSnapshots;
pub use sync_log::Entity as SyncLogs;
