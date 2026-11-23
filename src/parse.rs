use crate::song::{Chart, Header, NoteRow, NoteType, Song, TimingEngine};
use anyhow::Result;

/// Removes comments from text
fn strip_comments(s: &str) -> String {
    s.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
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
    for (measure_idx, measure_text) in note_text.split(",").enumerate() {
        // Split out rows within a measure
        let rows: Vec<&str> = measure_text
            .lines()
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .collect();

        let quantization = u8::try_from(rows.len()).unwrap();
        let measure = u16::try_from(measure_idx).unwrap();

        for (i, row) in rows.iter().enumerate() {
            
            let beat = (measure_idx as f64 * 4.0) + 4.0 * i as f64 / quantization as f64;
            let notes: [NoteType; 4] = row
                .chars()
                .map(NoteType::try_from)
                .collect::<Result<Vec<NoteType>>>()?
                .try_into()
                .map_err(|v: Vec<NoteType>| anyhow::anyhow!("Expected 4 columns, got {}", v.len()))?;

            chart.notes.push(NoteRow {
                notes: notes,
                measure: measure,
                quantization: quantization,
                beat: beat,
                seconds: timing_engine.beats_to_seconds(beat),
            });
        }
    }
    Ok(())
}

/// Parse the raw text content from a `.ssc` file
pub fn parse_simfile(content: String) -> Result<Song> {
    // Declare containers
    let mut song = Song::default();
    let mut header = Header::default();

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
