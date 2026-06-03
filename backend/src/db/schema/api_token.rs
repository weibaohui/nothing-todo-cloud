use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_name = "user_id")]
    pub user_id: i64,
    pub name: String,
    #[sea_orm(column_name = "token_hash")]
    pub token_hash: String,
    #[sea_orm(column_name = "last_used_at")]
    pub last_used_at: Option<DateTimeUtc>,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}