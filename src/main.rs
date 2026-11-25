use anyhow::{Context, Result};
use clap::Parser;
use rustmania::parse::parse;
use rustmania::serialize::save;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the simfile you wish to parse. Only `.ssc` is supported
    #[arg(short, long)]
    path: PathBuf,

    /// Print additional detailed output
    #[arg(short, long)]
    verbose: bool,

    /// Serialize to disk. Output to same directory as input.
    #[arg(short, long)]
    serialize: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let timer = Instant::now();

    let song = parse(&cli.path)?;
    println!(
        "Parsing completed in {:.2} ms",
        timer.elapsed().as_secs_f64() * 1000.0
    );

    if cli.serialize {
        save(&song, &cli.path).context("Failed to serialize to disk")?;
    }

    Ok(())
}
