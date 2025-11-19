pub use sea_orm_migration::prelude::*;

mod m20250627_201009_uploads_table;
mod m20251119_180646_mailing_list;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250627_201009_uploads_table::Migration),
            Box::new(m20251119_180646_mailing_list::Migration),
        ]
    }
}
