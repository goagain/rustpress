use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create update_updated_at_column() function (if not exists)
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_updated_at_column()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ language 'plpgsql';
                "#,
            )
            .await?;

        // 2. Create users table (directly use bigint as id)
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Role).string().not_null())
                    .col(ColumnDef::new(Users::Salt).string().not_null())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::ArchivedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Users::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Create indexes for users table
        manager
            .create_index(
                Index::create()
                    .name("idx_users_role")
                    .table(Users::Table)
                    .col(Users::Role)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_created_at")
                    .table(Users::Table)
                    .col(Users::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // 4. Create triggers for users table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS update_users_updated_at ON users;
                CREATE TRIGGER update_users_updated_at
                    BEFORE UPDATE ON users
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        // 5. Create posts table (directly use author_id as bigint, and establish foreign key)
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Posts::Title).string().not_null())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::Category).string().null())
                    .col(ColumnDef::new(Posts::AuthorId).big_integer().not_null())
                    .col(ColumnDef::new(Posts::Description).string().null())
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Posts::ArchivedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Posts::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 6. Create foreign key constraints for posts table
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_posts_author_id")
                    .from(Posts::Table, Posts::AuthorId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // 7. Create indexes for posts table
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_category")
                    .table(Posts::Table)
                    .col(Posts::Category)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_posts_created_at")
                    .table(Posts::Table)
                    .col(Posts::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_posts_author_id")
                    .table(Posts::Table)
                    .col(Posts::AuthorId)
                    .to_owned(),
            )
            .await?;

        // 8. Create triggers for posts table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS update_posts_updated_at ON posts;
                CREATE TRIGGER update_posts_updated_at
                    BEFORE UPDATE ON posts
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop triggers for posts table
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_posts_updated_at ON posts;")
            .await?;

        // Drop indexes for posts table
        manager
            .drop_index(Index::drop().name("idx_posts_author_id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_posts_created_at").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_posts_category").to_owned())
            .await?;

        // Drop foreign key constraints for posts table
        manager
            .drop_foreign_key(ForeignKey::drop().name("fk_posts_author_id").to_owned())
            .await?;

        // Drop posts table
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;

        // Drop triggers for users table
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_users_updated_at ON users;")
            .await?;

        // Drop indexes for users table
        manager
            .drop_index(Index::drop().name("idx_users_created_at").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_users_role").to_owned())
            .await?;

        // Drop users table
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        // Drop function
        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS update_updated_at_column();")
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    Role,
    Salt,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
    ArchivedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
    Title,
    Content,
    Category,
    AuthorId,
    Description,
    CreatedAt,
    UpdatedAt,
    ArchivedAt,
    DeletedAt,
}
