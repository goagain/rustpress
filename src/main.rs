mod api;
mod auth;
mod dto;
mod entity;
mod plugin;
mod repository;
mod rpk;
mod seed;
mod storage;

use api::create_router;
use repository::{PostRepository, PostgresPostRepository, PostgresUserRepository, UserRepository};
use sea_orm::{Database, DbErr};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize color-eyre for better error reporting with stack traces
    color_eyre::install().expect("Failed to install color-eyre");

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging with file location and line number
    // Log level can be controlled via RUST_LOG environment variable
    // Examples: RUST_LOG=info, RUST_LOG=debug, RUST_LOG=trace
    // Default is "info" if RUST_LOG is not set, but sqlx queries are set to debug level
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        .add_directive("sqlx=debug".parse().unwrap());

    tracing_subscriber::fmt()
        .with_file(true) // Show file path
        .with_line_number(true) // Show line number
        .with_target(true) // Show module path
        .with_thread_ids(false) // Hide thread IDs for cleaner output
        .with_thread_names(false) // Hide thread names for cleaner output
        .with_env_filter(filter) // Apply log level filter
        .init();

    // Get database URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/rustpress".to_string());

    // Connect to PostgreSQL database
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL database");

    tracing::info!("✅ Connected to PostgreSQL database");

    // Run SeaORM migrations
    run_migrations(&db)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("✅ Database migrations completed");

    // Create PostgreSQL Repositories
    let post_repository = PostgresPostRepository::new(db.clone());
    let user_repository = PostgresUserRepository::new(db.clone());

    // Initialize root user
    let root_user_id = init_root_user(&user_repository).await;

    // Print root password for testing (only if it was auto-generated)
    if let Ok(password) = std::env::var("ROOT_PASSWORD") {
        tracing::info!(
            "🔑 Root user password (from ROOT_PASSWORD env var): {}",
            password
        );
    } else if root_user_id.is_some() {
        tracing::info!(
            "🔑 Root user password was auto-generated. Set ROOT_PASSWORD env var to override."
        );
    }

    // Initialize sample post (if no posts exist and root user exists)
    if let Some(user_id) = root_user_id {
        init_sample_post(&post_repository, user_id).await;
    }

    // Create application state (contains Post and User Repository)
    let app_state = Arc::new(api::post_controller::AppState::new(
        post_repository,
        user_repository,
    ));

    // Create AI helper for plugins
    let ai_helper = Arc::new(plugin::ai::AiHelper::new(Arc::new(db.clone())));

    // Create post repository for plugins
    let postgres_post_repo = Arc::new(repository::PostgresPostRepository::new(db.clone()));

    let plugin_engine = Arc::new(plugin::engine::PluginEngine::new(
        Some(ai_helper.clone()),
        Arc::new(db.clone()),
    )?);

    // Initialize plugin manager
    let plugin_registry = Arc::new(plugin::registry::PluginRegistry::new(
        plugin::PluginManager::new(Arc::new(db.clone()))
            .expect("Failed to create plugin manager")
            .with_ai_helper(Arc::clone(&ai_helper))
            .with_post_repo(postgres_post_repo),
    ));

    let plugin_executer = Arc::new(plugin::registry::PluginExecuter::new(
        plugin_registry.clone(),
        Some(ai_helper.clone()),
        Arc::new(db.clone()),
    ));

    tracing::info!("✅ Plugin system initialized");

    // Initialize storage backend (local filesystem for now, can be switched to S3 later)
    let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "uploads".to_string());
    let storage_base_url = std::env::var("STORAGE_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3000/uploads".to_string());

    let storage = Arc::new(storage::LocalStorage::new(
        &storage_dir,
        storage_base_url.clone(),
    ));

    // Ensure storage directory exists
    storage
        .ensure_directory()
        .await
        .expect("Failed to create storage directory");

    tracing::info!("✅ Storage initialized");
    tracing::info!("   Directory: {}", storage_dir);
    tracing::info!("   Base URL: {}", storage_base_url);

    // Create routes (API Controller layer)
    let app = create_router(app_state, storage, db, plugin_registry, plugin_executer);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to 0.0.0.0:3000");

    tracing::info!("🚀 RustPress API server is running!");
    tracing::info!("📍 API server at:");
    tracing::info!("   http://localhost:3000");
    tracing::info!("   http://127.0.0.1:3000");
    tracing::info!("");
    tracing::info!("📝 API endpoints:");
    tracing::info!("   GET    /api/health        - Health check");
    tracing::info!("   GET    /api/posts         - Get all posts");
    tracing::info!("   GET    /api/posts/:id     - Get post by id");
    tracing::info!("   POST   /api/posts         - Create new post");
    tracing::info!("   PUT    /api/posts/:id     - Update post");
    tracing::info!("   DELETE /api/posts/:id     - Delete post");
    tracing::info!("   GET    /api/users         - Get all users");
    tracing::info!("   GET    /api/users/:id     - Get user by id");
    tracing::info!("   POST   /api/users         - Create new user");
    tracing::info!("   PUT    /api/users/:id     - Update user");
    tracing::info!("   DELETE /api/users/:id     - Delete user");
    tracing::info!("   POST   /api/auth/login    - User login");
    tracing::info!("   POST   /api/auth/refresh  - Refresh access token");
    tracing::info!("   POST   /api/upload/image  - Upload image");
    tracing::info!("");
    tracing::info!("📚 API Documentation:");
    tracing::info!("   Swagger UI: http://localhost:3000/swagger-ui");
    tracing::info!("   OpenAPI JSON: http://localhost:3000/api-doc/openapi.json");
    tracing::info!("");
    tracing::info!("💡 Frontend should run on http://localhost:5173 (Vite dev server)");

    axum::serve(listener, app).await.expect("Server error");
}

/// Run SeaORM migrations
async fn run_migrations(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm_migration::MigratorTrait;
    rustpress_migration::Migrator::up(db, None).await
}

/// Initialize root user
/// Creates a root user if it doesn't exist
/// Password is read from ROOT_PASSWORD environment variable, or randomly generated if not set
/// Returns the root user's ID (whether newly created or already existing)
async fn init_root_user<UR: UserRepository>(user_repository: &UR) -> Option<i64> {
    use crate::dto::{CreateUserRequest, UserRole};
    use uuid::Uuid;

    // Check if root user already exists (find user with Root role)
    let root_user = user_repository
        .find_by_username("root")
        .await
        .unwrap_or(None);

    if let Some(existing_user) = root_user {
        tracing::info!("✅ Root user already exists (ID: {})", existing_user.id);
        return Some(existing_user.id);
    }

    // Read password from environment variable, or generate random password if not set
    let (password, is_password_from_env) = match std::env::var("ROOT_PASSWORD") {
        Ok(pwd) => (pwd, true),
        Err(_) => {
            // Generate random password (32 characters, using two UUIDs concatenated and removing hyphens)
            let random_password = Uuid::new_v4().to_string() + &Uuid::new_v4().to_string();
            (random_password.replace("-", ""), false)
        }
    };

    // Create root user
    let create_request = CreateUserRequest {
        username: "root".to_string(),
        email: "root@rustpress.local".to_string(),
        password: password.clone(),
        role: UserRole::Root,
    };

    match user_repository.create(create_request).await {
        Ok(user) => {
            tracing::info!("✅ Root user created successfully");
            tracing::info!("   Username: {}", user.username);
            tracing::info!("   Email: {}", user.email);
            tracing::info!("   User ID: {}", user.id);

            if !is_password_from_env {
                tracing::warn!("");
                tracing::warn!("⚠️  Root password was randomly generated:");
                tracing::warn!("   Password: {}", password);
                tracing::warn!("");
                tracing::warn!("   Please save this password securely!");
                tracing::warn!(
                    "   You can set ROOT_PASSWORD environment variable to use a custom password."
                );
                tracing::warn!("");
            } else {
                tracing::info!("   Password: [from ROOT_PASSWORD environment variable]");
            }

            Some(user.id)
        }
        Err(e) => {
            tracing::error!("❌ Failed to create root user: {}", e);
            None
        }
    }
}

/// Initialize sample post
/// Creates a sample post if no posts exist
async fn init_sample_post<PR: PostRepository>(post_repository: &PR, author_id: i64) {
    use crate::dto::CreatePostRequest;

    // Check if posts already exist
    let posts = match post_repository.find_all().await {
        Ok(posts) => posts,
        Err(e) => {
            tracing::error!("❌ Failed to check existing posts: {}", e);
            return;
        }
    };

    if !posts.is_empty() {
        tracing::info!("✅ Posts already exist, skipping sample post creation");
        return;
    }

    // Create sample post
    let sample_post = seed::get_sample_post();
    let create_request = CreatePostRequest {
        title: sample_post.title,
        content: sample_post.content,
        category: Some(sample_post.category),
        author_id,
    };

    match post_repository.create(create_request).await {
        Ok(post) => {
            tracing::info!("✅ Sample post created successfully");
            tracing::info!("   Title: {}", post.title);
            tracing::info!("   Category: {:?}", post.category);
            tracing::info!("   Post ID: {}", post.id);
        }
        Err(e) => {
            tracing::error!("❌ Failed to create sample post: {}", e);
        }
    }
}
