use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Mail::Table)
                    .if_not_exists()
                    .col(pk_auto(Mail::Id))
                    .col(string(Mail::Email))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mail::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Mail {
    #[sea_orm(iden = "mailing_list")]
    Table,
    Id,
    Email,
}
