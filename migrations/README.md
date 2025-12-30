# SeaORM Migrations

本项目使用 SeaORM 管理数据库 migrations。

## Migration 文件位置

Migrations 定义在 `src/migration/` 目录中：
- `mod.rs` - Migration 模块定义
- `m20241225_000001_create_posts_table.rs` - 创建 posts 表的 migration

## 运行 Migrations

### 自动运行（推荐）

服务器启动时会自动运行所有 pending migrations。

### 手动运行

如果需要手动运行 migrations，可以使用 SeaORM CLI：

```bash
# 安装 SeaORM CLI（如果还没有）
cargo install sea-orm-cli

# 运行 migrations
sea-orm migrate up

# 回滚最后一个 migration
sea-orm migrate down

# 查看 migration 状态
sea-orm migrate status
```

## Migration 命名规则

SeaORM migrations 使用时间戳命名格式：
- `m{YYYYMMDD}_{HHMMSS}_{description}.rs`

例如：`m20241225_000001_create_posts_table.rs`

## 创建新的 Migration

1. 在 `src/migration/` 目录创建新文件
2. 实现 `MigrationTrait`
3. 在 `src/migration/mod.rs` 中注册新的 migration

## 注意事项

- Migrations 会在服务器启动时自动运行
- 确保数据库连接正常
- 生产环境建议手动管理 migrations
