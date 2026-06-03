use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "device_snapshots")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_name = "device_id")]
    pub device_id: i64,
    pub version: i64,
    #[sea_orm(column_name = "data_type")]
    pub data_type: String,
    #[sea_orm(column_name = "data_payload")]
    pub data_payload: String,
    pub checksum: String,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeUtc,
    pub metadata: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}