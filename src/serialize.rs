use crate::song::Song;
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub fn save(song: &Song, path: &Path) -> Result<()> {
    let mut output_path = PathBuf::from(path);
    output_path.set_extension("json");

    let writer = BufWriter::new(File::create(output_path)?);
    serde_json::to_writer_pretty(writer, song)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Song> {
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}
