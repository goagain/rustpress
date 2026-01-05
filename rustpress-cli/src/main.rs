use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;

#[derive(Parser)]
#[command(name = "rustpress-cli")]
#[command(about = "CLI tool for Rustpress CMS")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Plugin management commands
    Plugin(commands::plugin::PluginArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Plugin(plugin_args) => {
            commands::plugin::handle_plugin_command(&plugin_args)?;
        }
    }

    Ok(())
}
