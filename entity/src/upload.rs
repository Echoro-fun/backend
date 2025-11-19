use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::Metadata;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "uploads_table")]
pub struct Model {
    #[sea_orm(primary_key, unique, indexed, auto_increment = true)]
    pub id: i32,
    pub name: String,
    pub symbol: String,
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
