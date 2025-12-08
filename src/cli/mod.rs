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
    /// Initialize a new showfile
    Init {
        /// Path to create the showfile JSON
        #[arg(default_value = "showfile.json")]
        showfile_path: PathBuf,
    },
    /// Run the showfile
    Run {
        /// Path to the showfile JSON
        showfile_path: PathBuf,
    },
}

pub fn parse_and_execute() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { showfile_path } => {
            init::init_showfile(showfile_path)?;
        }
        Commands::Run { showfile_path } => {
            run::run_showfile(showfile_path)?;
        }
    }

    Ok(())
}
