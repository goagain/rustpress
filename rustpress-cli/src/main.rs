use clap::{Parser, Subcommand};
use colored::Colorize;
use std::env;

mod cli;
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

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
fn run() -> anyhow::Result<()> {
    // 1. get all arguments
    let mut args: Vec<String> = env::args().collect();

    // remove the first argument if it is "rustpress"
    if args.len() > 1 && args[1] == "rustpress" {
        args.remove(1);
    }

    // 2. parse arguments
    let cli = Cli::parse_from(args);

    // 3. route dispatch
    match cli.command {
        Commands::Plugin(plugin_args) => {
            commands::plugin::handle_plugin_command(&plugin_args)?;
        }
    }

    Ok(())
}
