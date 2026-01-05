use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(name = "rustpress")]
#[command(about = "CLI tool for Rustpress CMS plugin development")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// Create a new Wasm plugin from template
    New {
        /// The name of the plugin (snake_case recommended)
        name: String,
    },

    /// Build the plugin into a .wasm (and optionally .rpk) file
    Build {
        /// Build in release mode
        #[arg(long, short)]
        release: bool,
    },

    Pack {
        #[arg(long, short)]
        release: bool,
    },
}
