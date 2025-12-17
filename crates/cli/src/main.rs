use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod info;
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
    /// Initialize a new showfile.
    Init {
        /// Path to create the showfile at.
        showfile_path: PathBuf,
    },
    /// Run the showfile.
    Run {
        /// Path to the showfile.
        showfile_path: PathBuf,
    },
    /// Get info about a showfile.
    Info {
        #[command(subcommand)]
        command: InfoSubcommand,
    },
}

#[derive(Subcommand)]
enum InfoSubcommand {
    /// Dump the patch tree.
    Patch {
        /// Path to the showfile.
        showfile_path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let is_debug_mode = cfg!(debug_assertions);
    let default_level =
        if is_debug_mode { log::LevelFilter::Debug } else { log::LevelFilter::Info };
    pretty_env_logger::formatted_builder().filter_level(default_level).parse_env("RUST_LOG").init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { showfile_path } => {
            init::init_showfile(showfile_path)?;
        }
        Commands::Run { showfile_path } => {
            run::run_showfile(showfile_path)?;
        }
        Commands::Info { command: InfoSubcommand::Patch { showfile_path } } => {
            info::dump_patch(showfile_path)?;
        }
    }

    Ok(())
}
