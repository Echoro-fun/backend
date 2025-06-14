use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mailing_list")]
pub struct Model {
    #[sea_orm(primary_key, unique, indexed, auto_increment = true)]
    pub id: i32,
    #[sea_orm(unique)]
    pub email: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
