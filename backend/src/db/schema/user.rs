use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique, length = 255)]
    pub email: String,
    #[sea_orm(column_name = "password_hash")]
    pub password_hash: String,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeUtc,
    pub plan: String,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}