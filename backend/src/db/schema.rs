//! 数据库 Schema 定义
//! 使用 SeaORM 定义所有数据表结构

pub mod user;
pub mod device;
pub mod api_token;
pub mod device_snapshot;
pub mod sync_log;

pub use user::Entity as Users;
pub use device::Entity as Devices;
pub use api_token::Entity as ApiTokens;
pub use device_snapshot::Entity as DeviceSnapshots;
pub use sync_log::Entity as SyncLogs;
