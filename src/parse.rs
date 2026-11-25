use crate::serialize::load;
use crate::song::{Chart, Header, NoteRow, NoteType, Provenance, Song};
use crate::timing::TimingEngine;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

/// Parse the provided file
pub fn parse(source: &Path) -> Result<Song> {
    let extension = source
        .extension()
        .and_then(|e| e.to_str())
        .with_context(|| format!("No file extension: {}", source.display()))?;

    match extension {
        "json" => load(source),
        "ssc" => parse_simfile(source),
        other => bail!("Unknown file extension: {other}"),
    }
}

/// Parse the raw text content from a `.ssc` file
fn parse_simfile(source: &Path) -> Result<Song> {
    // Declare containers
    let mut song = Song::default();
    let mut header = Header::default();
    let content = fs::read_to_string(source)?;

    // Capture parser metadata
    song.provenance.source = source.to_string_lossy().into_owned();
    song.provenance.version = Provenance::version();
    song.provenance.hash = Provenance::hash(content.as_bytes());

    // Clean input text and split into sections
    let clean_content = strip_comments(&content);
    drop(content);
    let mut sections = clean_content.split("#NOTEDATA:");

    // Extract song header data
    let header_data = sections.next().unwrap_or_default();
    for (k, v) in get_key_values(header_data) {
        header.set(k, v)?;
    }

    if header.bpms.is_empty() {
        bail!("Missing or empty #BPMS");
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
                "NOTES" => parse_notes(&timing, &mut chart, v).with_context(|| {
                    format!(
                        "Chart: {:?}({:?})",
                        song.header.title.as_ref(),
                        chart.metadata.difficulty
                    )
                })?,
                _ => chart.metadata.set(k, v)?,
            }
        }
        song.charts.push(chart);
    }

    Ok(song)
}

/// Removes comments from text
fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(line.split_once("//").map_or(line, |(code, _)| code));
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

/// Parse an individual note row
fn parse_row(row: &str) -> Result<[NoteType; 4]> {
    let cols = row.chars().count();
    if cols != 4 {
        bail!("Expected 4 columns, got {cols}");
    }

    let mut notes = [NoteType::Empty; 4];
    for (note, c) in notes.iter_mut().zip(row.chars()) {
        *note = NoteType::try_from(c)?;
    }

    Ok(notes)
}

/// Parse chart note data
///
/// Measures are separated by "," delimiters. Q
/// Quantization is derived from number of rows in a measure.
fn parse_notes(timing_engine: &TimingEngine, chart: &mut Chart, note_text: &str) -> Result<()> {
    // Upper bound on row count (also counts blank and separator lines)
    chart.notes.reserve(note_text.lines().count());

    for (measure_idx, measure_text) in note_text.split(",").enumerate() {
        // Non-empty rows within a measure. A closure rather than a collected `Vec` so it
        // can be iterated twice (once to count, once to parse) without allocating per measure
        let rows = || {
            measure_text
                .lines()
                .map(str::trim)
                .filter(|r| !r.is_empty())
        };

        let quantization = u8::try_from(rows().count())
            .with_context(|| format!("Measure {measure_idx} has more than 255 rows"))?;
        let measure = u16::try_from(measure_idx).context("Chart has too many measures")?;

        for (i, row) in rows().enumerate() {
            // Position within the measure; counts empty rows, which aren't stored below
            let slot = u8::try_from(i)?;
            let notes =
                parse_row(row).with_context(|| format!("Measure {measure_idx}: {row:?}"))?;

            // Skip empty rows
            if notes.iter().all(|&n| n == NoteType::Empty) {
                continue;
            }

            chart.notes.push(NoteRow {
                notes,
                measure,
                quantization,
                slot,
                seconds: timing_engine.beats_to_seconds(NoteRow::beat_of(
                    measure,
                    slot,
                    quantization,
                )),
            });
        }
    }

    chart.notes.shrink_to_fit();
    Ok(())
}

#[cfg(test)]
mod tests;
