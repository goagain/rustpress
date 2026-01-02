use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OpenaiApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OpenaiApiKeys::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(OpenaiApiKeys::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(OpenaiApiKeys::ApiKey).string().not_null())
                    .col(ColumnDef::new(OpenaiApiKeys::Endpoint).string().null())
                    .col(
                        ColumnDef::new(OpenaiApiKeys::IsDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(OpenaiApiKeys::DefaultModel).string().null())
                    .col(
                        ColumnDef::new(OpenaiApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(OpenaiApiKeys::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS update_openai_api_keys_updated_at ON openai_api_keys;
                CREATE TRIGGER update_openai_api_keys_updated_at
                    BEFORE UPDATE ON openai_api_keys
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS update_openai_api_keys_updated_at ON openai_api_keys;",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(OpenaiApiKeys::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum OpenaiApiKeys {
    Table,
    Id,
    Name,
    ApiKey,
    Endpoint,
    IsDefault,
    DefaultModel,
    CreatedAt,
    UpdatedAt,
}
