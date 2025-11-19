use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Upload::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Upload::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Upload::Name).string().not_null())
                    .col(ColumnDef::new(Upload::Symbol).string().not_null())
                    .col(
                        ColumnDef::new(Upload::AudioUri)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Upload::ImageUri)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Upload::MetadataUri)
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
            .drop_table(Table::drop().table(Upload::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Upload {
    #[sea_orm(iden = "uploads_table")]
    Table,
    Id,
    Name,
    Symbol,
    AudioUri,
    ImageUri,
    MetadataUri,
}
