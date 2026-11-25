use anyhow::{Context, Result};
use clap::Parser;
use rustmania::parse;
use rustmania::serialize::save;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the file to load: a `.ssc` simfile, or `.json` written by `--output`
    #[arg(short, long)]
    path: PathBuf,

    /// Print additional detailed output
    #[arg(short, long)]
    verbose: bool,

    /// Serialize to JSON. Defaults to the input path with a `.json` extension.
    #[arg(short, long, value_name = "PATH", num_args = 0..=1)]
    output: Option<Option<PathBuf>>,

    /// Overwrite the output file if it already exists
    #[arg(short, long, requires = "output")]
    force: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let timer = Instant::now();

    let song = parse(&cli.path)?;
    println!("Loaded in {:.2} ms", timer.elapsed().as_secs_f64() * 1000.0);

    if let Some(output) = cli.output {
        let out_path = output.unwrap_or_else(|| cli.path.with_extension("json"));
        save(&song, &out_path, cli.force)
            .with_context(|| format!("Failed to write {}", out_path.display()))?;
    }

    Ok(())
}
