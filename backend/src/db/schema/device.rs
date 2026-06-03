use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "devices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_name = "user_id")]
    pub user_id: i64,
    #[sea_orm(column_name = "device_name")]
    pub device_name: String,
    #[sea_orm(column_name = "device_key")]
    pub device_key: Option<String>,
    #[sea_orm(column_name = "last_seen_at")]
    pub last_seen_at: DateTimeUtc,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}