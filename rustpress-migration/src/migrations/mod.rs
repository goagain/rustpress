use sea_orm_migration::MigrationTrait;

mod m20251230_000001_init_schema;
mod m20251231_000002_add_versioning_and_drafts;
mod m20260101_000003_add_settings_and_plugins;
mod m20260102_000004_add_openai_api_keys;

pub fn migration_list() -> Vec<Box<dyn MigrationTrait + 'static>> {
    vec![
        Box::new(m20251230_000001_init_schema::Migration),
        Box::new(m20251231_000002_add_versioning_and_drafts::Migration),
        Box::new(m20260101_000003_add_settings_and_plugins::Migration),
        Box::new(m20260102_000004_add_openai_api_keys::Migration),
    ]
}
