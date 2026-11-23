use anyhow::Result;
use clap::Parser;
use rustmania::parse::parse_simfile;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the simfile you wish to parse. Only `.ssc` is supported
    #[arg(short, long)]
    path: PathBuf,

    /// Print additional detailed output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let content = fs::read_to_string(&cli.path)?;

    let song = parse_simfile(content)?;
    println!("{:#?}", &song);

    Ok(())
}
