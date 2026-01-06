use sea_orm_migration::MigrationTrait;

mod m20251230_000001_init_schema;
mod m20251231_000002_add_versioning_and_drafts;
mod m20260101_000003_add_settings_and_plugins;
mod m20260102_000004_add_openai_api_keys;
mod m20260103_000005_add_plugin_manifest;
mod m20260104_000006_add_plugin_permissions;
mod m20260105_000007_add_plugin_status;
mod m20260105_000010_add_description_to_post_drafts;
mod m20260106_000008_add_plugin_id;
mod m20260107_000009_add_granted_permissions;

pub fn migration_list() -> Vec<Box<dyn MigrationTrait + 'static>> {
    vec![
        Box::new(m20251230_000001_init_schema::Migration),
        Box::new(m20251231_000002_add_versioning_and_drafts::Migration),
        Box::new(m20260101_000003_add_settings_and_plugins::Migration),
        Box::new(m20260102_000004_add_openai_api_keys::Migration),
        Box::new(m20260103_000005_add_plugin_manifest::Migration),
        Box::new(m20260104_000006_add_plugin_permissions::Migration),
        Box::new(m20260105_000007_add_plugin_status::Migration),
        Box::new(m20260106_000008_add_plugin_id::Migration),
        Box::new(m20260107_000009_add_granted_permissions::Migration),
        Box::new(m20260105_000010_add_description_to_post_drafts::Migration),
    ]
}
