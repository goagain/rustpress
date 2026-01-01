pub use sea_orm_migration::prelude::*;

mod m20251230_000001_init_schema;
mod m20251231_000002_add_versioning_and_drafts;
mod m20260101_000003_add_settings_and_plugins;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251230_000001_init_schema::Migration),
            Box::new(m20251231_000002_add_versioning_and_drafts::Migration),
            Box::new(m20260101_000003_add_settings_and_plugins::Migration),
        ]
    }
}
