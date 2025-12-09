use anyhow::Context;

mod cli;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    cli::parse_and_execute()?;
    Ok(())
}
