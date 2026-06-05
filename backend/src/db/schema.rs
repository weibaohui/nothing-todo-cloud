//! 数据库 Schema 定义
//! 使用 SeaORM 定义所有数据表结构

pub mod user;
pub mod device;
pub mod api_token;
pub mod user_todo;
pub mod user_tag;
pub mod user_skill;
pub mod sync_log;

pub use user::Entity as Users;
pub use device::Entity as Devices;
pub use api_token::Entity as ApiTokens;
pub use user_todo::Entity as UserTodos;
pub use user_tag::Entity as UserTags;
pub use user_skill::Entity as UserSkills;
pub use sync_log::Entity as SyncLogs;
