use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add description column to post_drafts table
        manager
            .alter_table(
                Table::alter()
                    .table(PostDrafts::Table)
                    .add_column(ColumnDef::new(PostDrafts::Description).string().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove description column from post_drafts table
        manager
            .alter_table(
                Table::alter()
                    .table(PostDrafts::Table)
                    .drop_column(PostDrafts::Description)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum PostDrafts {
    Table,
    Description,
}
