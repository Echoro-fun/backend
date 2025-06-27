use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "uploads_table")]
pub struct Model {
    #[sea_orm(primary_key, unique, indexed, auto_increment = true)]
    pub id: i32,
    #[sea_orm(unique)]
    pub uri: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
