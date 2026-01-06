mod build;
mod new;
mod pack;

use anyhow::Result;
use clap::Args;

/// Plugin-related commands
#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(clap::Subcommand)]
pub enum PluginCommand {
    /// Create a new Rustpress plugin
    New {
        /// Name of the plugin to create
        plugin_name: String,
    },
    Build {
        /// Build in release mode
        #[arg(long, short)]
        release: bool,
    },
    Pack {
        #[arg(long, short)]
        release: bool,

        /// Optional output directory for the plugin package
        #[arg(long)]
        output_dir: Option<String>,
    },
}

pub fn handle_plugin_command(args: &PluginArgs) -> Result<()> {
    match &args.command {
        PluginCommand::New { plugin_name } => new::create_new_plugin(plugin_name),
        PluginCommand::Build { release } => {
            build::build_plugin(release, std::path::Path::new("."))?;
            Ok(())
        }
        PluginCommand::Pack {
            release,
            output_dir,
        } => pack::pack_plugin(release, output_dir),
    }
}
