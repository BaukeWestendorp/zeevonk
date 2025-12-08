mod cli;

fn main() -> anyhow::Result<()> {
    cli::parse_and_execute()?;
    Ok(())
}
