use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add granted_permissions column to plugins table
        manager
            .alter_table(
                Table::alter()
                    .table(Plugins::Table)
                    .add_column(
                        ColumnDef::new(Plugins::GrantedPermissions)
                            .json()
                            .default(Expr::val(serde_json::json!([]))),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop column
        manager
            .alter_table(
                Table::alter()
                    .table(Plugins::Table)
                    .drop_column(Plugins::GrantedPermissions)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Plugins {
    Table,
    GrantedPermissions,
}
