use crate::song::{Chart, Header, NoteRow, NoteType, Song, TimingEngine};
use crate::serialize::load;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::Path;
use std::fs;


pub fn parse(source: &Path) -> Result<Song> {
    let extension = source.extension().unwrap().to_str().unwrap();
    let song = match extension {
        "json" => load(source).unwrap(),
        "ssc" => parse_simfile(source).unwrap(),
        other => bail!("Unknown file extension: {other}"),
    };
    Ok(song)
}

/// Parse the raw text content from a `.ssc` file
fn parse_simfile(source: &Path) -> Result<Song> {
    // Declare containers
    let mut song = Song::default();
    let mut header = Header::default();
    let content = fs::read_to_string(source)?;

    // Capture parser metadata
    song.parser.source = String::from(source.to_str().unwrap());
    song.parser.version = Parser::version();
    song.parser.hash = Parser::hash(content.as_bytes());

    // Clean input text and split into sections
    let clean_content = strip_comments(&content);
    drop(content);
    let mut sections = clean_content.split("#NOTEDATA:");

    // Extract song header data
    let header_data = sections.next().unwrap_or_default();
    for (k, v) in get_key_values(header_data) {
        header.set(k, v)?;
    }
    song.header = header;

    // Extract metadata and notes from chart(s)
    let timing = TimingEngine::new(
        &song.header.bpms,
        &song.header.stops,
        song.header.offset.unwrap_or(0.0),
    );
    for chart_text in sections {
        let mut chart = Chart::default();
        for (k, v) in get_key_values(chart_text) {
            match k {
                "NOTES" => parse_notes(&timing, &mut chart, v)?,
                _ => chart.metadata.set(k, v)?,
            }
        }
        song.charts.push(chart);
    }

    Ok(song)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Parser {
    pub source: String,
    pub version: String,
    pub hash: String,
}

impl Parser {
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    pub fn hash(content: &[u8]) -> String {
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(content);
            let result = hasher.finalize();
            result.iter().map(|b| format!("{:02x}", b)).collect()
        };
        hash
    }
}

/// Removes comments from text
fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(line.split("//").next().unwrap_or(""));
    }
    out
}

/// Extract key:value pairs
///
/// Data is of the form `#KEY:VALUE;`
fn get_key_values(s: &str) -> impl Iterator<Item = (&str, &str)> {
    s.split(';').filter_map(|chunk| {
        let (k, v) = chunk.trim().split_once(':')?;
        Some((k.strip_prefix('#').unwrap_or(k), v))
    })
}

/// Parse chart note data
/// 
/// Measures are separated by "," delimiters. Quantization is derived from number of rows in a measure.
fn parse_notes(timing_engine: &TimingEngine, chart: &mut Chart, note_text: &str) -> Result<()> {
    // Upper bound on row count (also counts blank and separator lines)
    chart.notes.reserve(note_text.lines().count());

    for (measure_idx, measure_text) in note_text.split(",").enumerate() {
        // Split out rows within a measure
        let rows = || {
            measure_text
                .lines()
                .map(str::trim)
                .filter(|r| !r.is_empty())
        };

        let quantization = u8::try_from(rows().count()).unwrap();
        let measure = u16::try_from(measure_idx).unwrap();

        for (i, row) in rows().enumerate() {
            // Position within the measure; counts empty rows, which aren't stored below
            let slot = u8::try_from(i).unwrap();

            // Parse the individual row columns
            let mut notes: [NoteType; 4] = Default::default();
            let mut cols = 0;
            for c in row.chars() {
                if cols == 4 {
                    anyhow::bail!("Expected 4 columns, got more");
                }
                notes[cols] = NoteType::try_from(c)?;
                cols += 1;
            }
            if cols != 4 {
                anyhow::bail!("Expected 4 columns, got {cols}");
            }

            // Rows are validated above, but empty ones aren't stored: slot/seconds are
            // kept on every stored row, so the timeline can be inferred without them.
            if notes.iter().all(|n| matches!(n, NoteType::Empty)) {
                continue;
            }

            chart.notes.push(NoteRow {
                notes: notes,
                measure: measure,
                quantization: quantization,
                slot: slot,
                seconds: timing_engine
                    .beats_to_seconds(NoteRow::beat_of(measure, slot, quantization)),
            });
        }
    }
    
    chart.notes.shrink_to_fit();
    Ok(())
}

