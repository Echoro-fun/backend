use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "uploads_table")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub symbol: String,
    pub decimal: u8,
    #[sea_orm(unique)]
    pub audio_uri: String,
    #[sea_orm(unique)]
    pub image_uri: String,
    #[sea_orm(unique)]
    pub metadata_uri: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
