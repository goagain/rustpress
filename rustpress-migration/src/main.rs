use clap::{Parser, Subcommand};
use sea_orm::{Database, DbErr};
use sea_orm_migration::MigratorTrait;
use std::process;

pub use sea_orm_migration::prelude::*;

mod lib;
pub use lib::Migrator;

#[derive(Parser)]
#[command(name = "rustpress-migrate")]
#[command(about = "RustPress database migration tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database URL (overrides DATABASE_URL environment variable)
    #[arg(short, long, global = true)]
    database_url: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all pending migrations
    Up {
        /// Number of migrations to run (default: all)
        #[arg(short, long)]
        steps: Option<u32>,
    },
    /// Rollback migrations
    Down {
        /// Number of migrations to rollback (default: 1)
        #[arg(short, long, default_value = "1")]
        steps: u32,
    },
    /// Show migration status
    Status,
    /// Rollback all migrations
    Reset,
    /// Rollback all migrations and re-run them
    Refresh,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Load environment variables
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // Get database URL
    let database_url = cli.database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| {
            eprintln!("❌ DATABASE_URL not set. Please set it via environment variable or --database-url flag.");
            process::exit(1);
        });

    // Connect to database
    let db = match Database::connect(&database_url).await {
        Ok(db) => {
            tracing::info!("✅ Connected to database");
            db
        }
        Err(e) => {
            tracing::error!("❌ Failed to connect to database: {}", e);
            process::exit(1);
        }
    };

    // Execute command
    let result = match cli.command {
        Commands::Up { steps } => {
            tracing::info!("🔄 Running migrations...");
            if let Some(steps) = steps {
                Migrator::up(&db, Some(steps)).await
            } else {
                Migrator::up(&db, None).await
            }
        }
        Commands::Down { steps } => {
            tracing::info!("⬇️  Rolling back {} migration(s)...", steps);
            Migrator::down(&db, Some(steps)).await
        }
        Commands::Status => {
            tracing::info!("📊 Migration Status:");
            tracing::info!("   Status command not implemented yet");
            return;
        }
        Commands::Reset => {
            tracing::info!("🔄 Resetting database (rolling back all migrations)...");
            Migrator::reset(&db).await
        }
        Commands::Refresh => {
            tracing::info!("🔄 Refreshing database (reset + migrate)...");
            Migrator::refresh(&db).await
        }
    };

    match result {
        Ok(_) => {
            tracing::info!("✅ Migration completed successfully");
        }
        Err(e) => {
            tracing::error!("❌ Migration failed: {}", e);
            process::exit(1);
        }
    }
}

// async fn show_status(db: &sea_orm::DatabaseConnection) {
//     // TODO: Implement migration status checking when sea-orm-migration API is clarified
// }
