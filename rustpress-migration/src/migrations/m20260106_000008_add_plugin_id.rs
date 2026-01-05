use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add plugin_id column to plugins table
        manager
            .alter_table(
                Table::alter()
                    .table(Plugins::Table)
                    .add_column(
                        ColumnDef::new(Plugins::PluginId)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on plugin_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_plugins_plugin_id")
                    .table(Plugins::Table)
                    .col(Plugins::PluginId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index
        manager
            .drop_index(Index::drop().name("idx_plugins_plugin_id").to_owned())
            .await?;

        // Drop column
        manager
            .alter_table(
                Table::alter()
                    .table(Plugins::Table)
                    .drop_column(Plugins::PluginId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Plugins {
    Table,
    PluginId,
}
