use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "user_todos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_name = "user_id")]
    pub user_id: i64,
    pub title: String,
    pub prompt: Option<String>,
    pub status: Option<String>,
    pub executor: Option<String>,
    #[sea_orm(column_name = "scheduler_enabled")]
    pub scheduler_enabled: Option<i32>,  // SQLite INTEGER: 0 or 1
    #[sea_orm(column_name = "scheduler_config")]
    pub scheduler_config: Option<String>,
    #[sea_orm(column_name = "tag_names")]
    pub tag_names: Option<String>,  // JSON array
    pub workspace: Option<String>,
    pub worktree: Option<String>,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeUtc,
    #[sea_orm(column_name = "updated_at")]
    pub updated_at: Option<DateTimeUtc>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
