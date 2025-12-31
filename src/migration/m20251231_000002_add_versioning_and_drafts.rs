use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create post_versions table for versioning
        manager
            .create_table(
                Table::create()
                    .table(PostVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostVersions::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(PostVersions::PostId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PostVersions::Title).string().not_null())
                    .col(ColumnDef::new(PostVersions::Content).text().not_null())
                    .col(ColumnDef::new(PostVersions::Category).string().not_null())
                    .col(
                        ColumnDef::new(PostVersions::VersionNumber)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostVersions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PostVersions::CreatedBy)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostVersions::ChangeNote)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create foreign key for post_versions
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_post_versions_post_id")
                    .from(PostVersions::Table, PostVersions::PostId)
                    .to(Posts::Table, Posts::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create indexes for post_versions
        manager
            .create_index(
                Index::create()
                    .name("idx_post_versions_post_id")
                    .table(PostVersions::Table)
                    .col(PostVersions::PostId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_versions_version_number")
                    .table(PostVersions::Table)
                    .col(PostVersions::VersionNumber)
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on (post_id, version_number)
        manager
            .create_index(
                Index::create()
                    .name("idx_post_versions_unique")
                    .table(PostVersions::Table)
                    .col(PostVersions::PostId)
                    .col(PostVersions::VersionNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create post_drafts table for draft caching
        manager
            .create_table(
                Table::create()
                    .table(PostDrafts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostDrafts::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(PostDrafts::PostId)
                            .big_integer()
                            .null(), // null for new posts, not null for editing existing posts
                    )
                    .col(ColumnDef::new(PostDrafts::Title).string().not_null())
                    .col(ColumnDef::new(PostDrafts::Content).text().not_null())
                    .col(ColumnDef::new(PostDrafts::Category).string().not_null())
                    .col(
                        ColumnDef::new(PostDrafts::AuthorId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostDrafts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PostDrafts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create foreign key for post_drafts (post_id)
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_post_drafts_post_id")
                    .from(PostDrafts::Table, PostDrafts::PostId)
                    .to(Posts::Table, Posts::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create foreign key for post_drafts (author_id)
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_post_drafts_author_id")
                    .from(PostDrafts::Table, PostDrafts::AuthorId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create indexes for post_drafts
        manager
            .create_index(
                Index::create()
                    .name("idx_post_drafts_post_id")
                    .table(PostDrafts::Table)
                    .col(PostDrafts::PostId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_drafts_author_id")
                    .table(PostDrafts::Table)
                    .col(PostDrafts::AuthorId)
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on (post_id, author_id) - one draft per post per author
        manager
            .create_index(
                Index::create()
                    .name("idx_post_drafts_unique")
                    .table(PostDrafts::Table)
                    .col(PostDrafts::PostId)
                    .col(PostDrafts::AuthorId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create trigger for post_drafts updated_at
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS update_post_drafts_updated_at ON post_drafts;
                CREATE TRIGGER update_post_drafts_updated_at
                    BEFORE UPDATE ON post_drafts
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop trigger for post_drafts
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_post_drafts_updated_at ON post_drafts;")
            .await?;

        // Drop indexes for post_drafts
        manager
            .drop_index(Index::drop().name("idx_post_drafts_unique").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_post_drafts_author_id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_post_drafts_post_id").to_owned())
            .await?;

        // Drop foreign keys for post_drafts
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_post_drafts_author_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_post_drafts_post_id")
                    .to_owned(),
            )
            .await?;

        // Drop post_drafts table
        manager
            .drop_table(Table::drop().table(PostDrafts::Table).to_owned())
            .await?;

        // Drop indexes for post_versions
        manager
            .drop_index(Index::drop().name("idx_post_versions_unique").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_post_versions_version_number").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_post_versions_post_id").to_owned())
            .await?;

        // Drop foreign key for post_versions
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_post_versions_post_id")
                    .to_owned(),
            )
            .await?;

        // Drop post_versions table
        manager
            .drop_table(Table::drop().table(PostVersions::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum PostVersions {
    Table,
    Id,
    PostId,
    Title,
    Content,
    Category,
    VersionNumber,
    CreatedAt,
    CreatedBy,
    ChangeNote,
}

#[derive(DeriveIden)]
enum PostDrafts {
    Table,
    Id,
    PostId,
    Title,
    Content,
    Category,
    AuthorId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
