use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod init;
mod run;

#[derive(Parser)]
#[command(name = "zeevonk")]
#[command(about = "The Zeevonk CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project.
    Init {
        /// Path to create the project at.
        project_path: PathBuf,
    },
    /// Run the project.
    Run {
        /// Path to the project.
        project_path: PathBuf,
    },
}

#[derive(Subcommand)]
enum InfoSubcommand {
    /// Dump the stage tree.
    Patch {
        /// Path to the project.
        project_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let is_debug_mode = cfg!(debug_assertions);
    let default_level =
        if is_debug_mode { log::LevelFilter::Debug } else { log::LevelFilter::Info };
    pretty_env_logger::formatted_builder().filter_level(default_level).parse_env("RUST_LOG").init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project_path } => {
            init::init_project(project_path)?;
        }
        Commands::Run { project_path } => {
            run::run_project(project_path).await?;
        }
    }

    Ok(())
}
