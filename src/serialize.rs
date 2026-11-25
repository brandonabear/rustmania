use crate::song::Song;
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Writes `song` as JSON to `path`.
///
/// Fails if `path` already exists, unless `force` is set.
pub fn save(song: &Song, path: &Path, force: bool) -> Result<()> {
    let file = if force {
        File::create(path)?
    } else {
        File::create_new(path)?
    };
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, song)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Song> {
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}
