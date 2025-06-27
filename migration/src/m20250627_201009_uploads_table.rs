use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Uploads::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Uploads::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Uploads::Uri)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Uploads::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Uploads {
    Table,
    Id,
    Uri,
}
