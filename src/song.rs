mod header;
pub use header::{BeatChange, Header};
mod chart;
pub use chart::{Chart, ChartMetadata, Difficulty, GrooveRadar};
mod notes;
pub use notes::{NoteRow, NoteType};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Represents a song (i.e. simfile)
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Song {
    pub provenance: Provenance,
    pub header: Header,
    pub charts: Vec<Chart>,
}

/// Captures parsing metadata
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Provenance {
    pub source: String,
    pub version: String,
    pub hash: String,
}

impl Provenance {
    pub(crate) fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    pub(crate) fn hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests;
