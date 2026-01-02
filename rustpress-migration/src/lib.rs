pub use sea_orm_migration::prelude::*;

#[path = "migrations/mod.rs"]
mod migrations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        migrations::migration_list()
    }
}
