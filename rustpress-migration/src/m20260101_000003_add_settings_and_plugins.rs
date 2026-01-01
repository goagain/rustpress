use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create settings table for system configuration
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Settings::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Settings::Value)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Settings::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Settings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Settings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create trigger for settings updated_at
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS update_settings_updated_at ON settings;
                CREATE TRIGGER update_settings_updated_at
                    BEFORE UPDATE ON settings
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        // Insert default settings
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                INSERT INTO settings (key, value, description)
                VALUES 
                    ('allow_external_registration', 'true', 'Whether external users can register'),
                    ('maintenance_mode', 'false', 'Whether the site is in maintenance mode')
                ON CONFLICT (key) DO NOTHING;
                "#,
            )
            .await?;

        // Create plugins table for plugin management (reserved for future use)
        manager
            .create_table(
                Table::create()
                    .table(Plugins::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Plugins::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(Plugins::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Plugins::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Plugins::Version)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plugins::Enabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Plugins::Config)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Plugins::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Plugins::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create trigger for plugins updated_at
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS update_plugins_updated_at ON plugins;
                CREATE TRIGGER update_plugins_updated_at
                    BEFORE UPDATE ON plugins
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        // Add banned_at column to users table for user banning
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::BannedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for banned users
        manager
            .create_index(
                Index::create()
                    .name("idx_users_banned_at")
                    .table(Users::Table)
                    .col(Users::BannedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index for users banned_at
        manager
            .drop_index(Index::drop().name("idx_users_banned_at").to_owned())
            .await?;

        // Drop banned_at column from users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::BannedAt)
                    .to_owned(),
            )
            .await?;

        // Drop trigger for plugins
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_plugins_updated_at ON plugins;")
            .await?;

        // Drop plugins table
        manager
            .drop_table(Table::drop().table(Plugins::Table).to_owned())
            .await?;

        // Drop trigger for settings
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_settings_updated_at ON settings;")
            .await?;

        // Drop settings table
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    Key,
    Value,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Plugins {
    Table,
    Id,
    Name,
    Description,
    Version,
    Enabled,
    Config,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    BannedAt,
}
