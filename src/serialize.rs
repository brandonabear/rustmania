use crate::song:: Song;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub fn save(song: &Song, in_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut output_path = PathBuf::from(in_path);
    output_path.set_extension("json");

    let writer = BufWriter::new(File::create(output_path)?);
    serde_json::to_writer_pretty(writer, song)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Song, Box<dyn std::error::Error>> {
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}