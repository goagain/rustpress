use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create plugin_permissions table for granular permission management
        manager
            .create_table(
                Table::create()
                    .table(PluginPermissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PluginPermissions::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(PluginPermissions::PluginId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PluginPermissions::Permission)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PluginPermissions::IsGranted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(PluginPermissions::PermissionType)
                            .string()
                            .not_null(), // "required" or "optional"
                    )
                    .col(
                        ColumnDef::new(PluginPermissions::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PluginPermissions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PluginPermissions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_plugin_permissions_plugin_id")
                    .table(PluginPermissions::Table)
                    .col(PluginPermissions::PluginId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_plugin_permissions_plugin_perm")
                    .table(PluginPermissions::Table)
                    .col(PluginPermissions::PluginId)
                    .col(PluginPermissions::Permission)
                    .to_owned(),
            )
            .await?;

        // Create trigger for updated_at
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS update_plugin_permissions_updated_at ON plugin_permissions;
                CREATE TRIGGER update_plugin_permissions_updated_at
                    BEFORE UPDATE ON plugin_permissions
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop trigger
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_plugin_permissions_updated_at ON plugin_permissions;")
            .await?;

        // Drop indexes
        manager
            .drop_index(Index::drop().name("idx_plugin_permissions_plugin_perm").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_plugin_permissions_plugin_id").to_owned())
            .await?;

        // Drop table
        manager
            .drop_table(Table::drop().table(PluginPermissions::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum PluginPermissions {
    Table,
    Id,
    PluginId,
    Permission,
    IsGranted,
    PermissionType, // "required" or "optional"
    Description,
    CreatedAt,
    UpdatedAt,
}