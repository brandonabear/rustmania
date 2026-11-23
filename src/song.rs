use anyhow::{Context, Result, bail};
use std::str::FromStr;

/// Represents the beat at which a time change is applied.
/// 
/// A `BeatChange` usually represents BPM changes or Stops.
#[derive(Debug, Clone, Copy)]
pub struct BeatChange {
    pub beat: f64,
    pub value: f64,
}

/// Enum of valid chart Difficulties
#[derive(Default, Debug)]
pub enum Difficulty {
    #[default]
    Beginner,
    Easy,
    Medium,
    Hard,
    Challenge,
    Edit,
}

impl FromStr for Difficulty {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Beginner" => Ok(Difficulty::Beginner),
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            "Challenge" => Ok(Difficulty::Challenge),
            "Edit" => Ok(Difficulty::Edit),
            other => bail!("Unknown Difficulty: {other}"),
        }
    }
}

/// Parses `beat=value,beat=value,...` into `(beat, value)` pairs in file order.
/// Returns `None` if any entry is malformed.
fn parse_pairs(s: &str) -> Option<Vec<(f64, f64)>> {
    s.split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (beat, value) = p.split_once('=')?;
            Some((beat.trim().parse().ok()?, value.trim().parse().ok()?))
        })
        .collect()
}

/// Parses raw beat change strings into a vector of `BeatChange` structs
fn parse_beat_change(s: &str) -> Option<Vec<BeatChange>> {
    Some(
        parse_pairs(s)?
            .into_iter()
            .map(|(beat, value)| BeatChange { beat, value })
            .collect(),
    )
}

/// Represents the song-level metadata of a simfile
#[derive(Debug, Default)]
pub struct Header {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub artist: Option<String>,
    pub titletranslit: Option<String>,
    pub subtitletranslit: Option<String>,
    pub artisttranslit: Option<String>,
    pub genre: Option<String>,
    pub credit: Option<String>,
    pub banner: Option<String>,
    pub background: Option<String>,
    pub lyricspath: Option<String>,
    pub cdtitle: Option<String>,
    pub music: Option<String>,
    pub samplestart: Option<f64>,
    pub samplelength: Option<f64>,
    pub displaybpm: Option<String>,
    pub selectable: Option<bool>,
    pub bgchanges: Option<String>,
    pub fgchanges: Option<String>,
    pub offset: Option<f64>,
    pub bpms: Vec<BeatChange>,
    pub stops: Vec<BeatChange>,
}

impl Header {
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let value = value.trim();
        match key {
            "TITLE" => self.title = Some(value.to_owned()),
            "SUBTITLE" => self.subtitle = Some(value.to_owned()),
            "ARTIST" => self.artist = Some(value.to_owned()),
            "TITLETRANSLIT" => self.titletranslit = Some(value.to_owned()),
            "SUBTITLETRANSLIT" => self.subtitletranslit = Some(value.to_owned()),
            "ARTISTTRANSLIT" => self.artisttranslit = Some(value.to_owned()),
            "GENRE" => self.genre = Some(value.to_owned()),
            "CREDIT" => self.credit = Some(value.to_owned()),
            "BANNER" => self.banner = Some(value.to_owned()),
            "BACKGROUND" => self.background = Some(value.to_owned()),
            "LYRICSPATH" => self.lyricspath = Some(value.to_owned()),
            "CDTITLE" => self.cdtitle = Some(value.to_owned()),
            "MUSIC" => self.music = Some(value.to_owned()),
            "SAMPLESTART" => self.samplestart = Some(value.parse::<f64>()?),
            "SAMPLELENGTH" => self.samplelength = Some(value.parse::<f64>()?),
            "DISPLAYBPM" => self.displaybpm = Some(value.to_owned()),
            "SELECTABLE" => self.selectable = Some(value.eq_ignore_ascii_case("YES")),
            "BGCHANGES" => self.bgchanges = Some(value.to_owned()),
            "FGCHANGES" => self.fgchanges = Some(value.to_owned()),
            "OFFSET" => self.offset = Some(value.parse::<f64>()?),
            "BPMS" => self.bpms = parse_beat_change(value).context("invalid #BPMS")?,
            "STOPS" => self.stops = parse_beat_change(value).context("invalid #STOPS")?,
            _ => {}
        }
        Ok(())
    }
}

/// Represents the information from a simfile that affects timing
pub struct TimingEngine {
    bpms: Vec<BeatChange>,
    stops: Vec<BeatChange>,
    offset: f64,
}

impl TimingEngine {
    pub fn new(bpms: &[BeatChange], stops: &[BeatChange], offset: f64) -> Self {
        // Sorted copy of BPM changes, with an infinite ceiling so the last segment is open-ended
        let mut bpms = bpms.to_vec();
        bpms.push(BeatChange {
            beat: f64::INFINITY,
            value: f64::NAN,
        });
        bpms.sort_by(|a, b| a.beat.total_cmp(&b.beat));

        let mut stops = stops.to_vec();
        stops.sort_by(|a, b| a.beat.total_cmp(&b.beat));

        Self {
            bpms,
            stops,
            offset,
        }
    }

    pub fn beats_to_seconds(&self, target_beat: f64) -> f64 {
        let mut elapsed = 0.0;

        // Elapsed time across BPM changes
        for bpm in self.bpms.windows(2) {
            let (cur, next) = (&bpm[0], &bpm[1]);
            if target_beat <= cur.beat {
                break;
            }
            if target_beat <= next.beat {
                elapsed += (target_beat - cur.beat) * (60.0 / cur.value);
                break;
            }
            elapsed += (next.beat - cur.beat) * (60.0 / cur.value);
        }

        // Stops that occur before the target beat
        for stop in &self.stops {
            if stop.beat < target_beat {
                elapsed += stop.value;
            } else {
                break;
            }
        }

        elapsed - self.offset
    }
}

/// Represents statistics that drive the GrooveRadar game element
#[derive(Debug, Default)]
pub struct GrooveRadar {
    pub stream: Option<f64>,
    pub voltage: Option<f64>,
    pub air: Option<f64>,
    pub freeze: Option<f64>,
    pub chaos: Option<f64>,
    pub steps: Option<u32>,
    pub taps: Option<u32>,
    pub jumps: Option<u32>,
    pub holds: Option<u32>,
    pub mines: Option<u32>,
    pub hands: Option<u32>,
    pub rolls: Option<u32>,
    pub lifts: Option<u32>,
    pub fakes: Option<u32>,
}

impl FromStr for GrooveRadar {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut values: Vec<&str> = s.split(',').map(str::trim).collect();

        // Two-player files repeat the block once per player; only read the first.
        match values.len() {
            22 => values.truncate(11),
            28 => values.truncate(14),
            _ => {}
        }

        if values.len() < 5 {
            bail!("Expected at least 5 values, got {}", values.len());
        }

        // Count fields beyond `chaos` are only present in the 11- and 14-value forms
        // (the 11-value form omits rolls, lifts, and fakes).
        let count = |i: usize| -> Result<Option<u32>> {
            match values.get(i) {
                Some(v) => Ok(Some(v.parse()?)),
                None => Ok(None),
            }
        };

        Ok(GrooveRadar {
            stream: Some(values[0].parse::<f64>()?),
            voltage: Some(values[1].parse::<f64>()?),
            air: Some(values[2].parse::<f64>()?),
            freeze: Some(values[3].parse::<f64>()?),
            chaos: Some(values[4].parse::<f64>()?),
            steps: count(5)?,
            taps: count(6)?,
            jumps: count(7)?,
            holds: count(8)?,
            mines: count(9)?,
            hands: count(10)?,
            rolls: count(11)?,
            lifts: count(12)?,
            fakes: count(13)?,
        })
    }
}

/// Represents the chart-level metadata of a simfile
#[derive(Debug, Default)]
pub struct ChartMetadata {
    pub mode: String,
    pub description: String,
    pub credit: String,
    pub chart_name: String,
    pub difficulty: Difficulty,
    pub rating: f64,
    pub radar: GrooveRadar,
}

impl ChartMetadata {
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let value = value.trim();
        match key {
            "STEPSTYPE" => self.mode = value.to_owned(),
            "DESCRIPTION" => self.description = value.to_owned(),
            "CREDIT" => self.credit = value.to_owned(),
            "CHARTNAME" => self.chart_name = value.to_owned(),
            "DIFFICULTY" => self.difficulty = value.parse::<Difficulty>().unwrap(),
            "METER" => self.rating = value.parse::<f64>()?,
            "RADARVALUES" => self.radar = value.parse::<GrooveRadar>()?,
            _ => {}
        }
        Ok(())
    }
}

/// Represents a given chart within a song
#[derive(Debug, Default)]
pub struct Chart {
    pub metadata: ChartMetadata,
    pub notes: Vec<NoteRow>,
}

/// Represents a song (i.e. simfile)
#[derive(Debug, Default)]
pub struct Song {
    pub header: Header,
    pub charts: Vec<Chart>,
}

/// Represents all possible panel values
#[derive(Default, Debug)]
pub enum NoteType {
    #[default]
    Empty,
    Tap,
    Hold,
    Tail,
    Roll,
    Mine,
    Fake,
    Lift,
}

impl TryFrom<char> for NoteType {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        Ok(match c {
            '0' => NoteType::Empty,
            '1' => NoteType::Tap,
            '2' => NoteType::Hold,
            '3' => NoteType::Tail,
            '4' => NoteType::Roll,
            'M' => NoteType::Mine,
            'F' => NoteType::Fake,
            'L' => NoteType::Lift,
            other => bail!("Unknown note character: {other:?}"),
        })
    }
}

/// Represents all possible directions within a column (panel)
#[derive(Debug, Clone, Copy)]
pub enum Column {
    Left,
    Down,
    Up,
    Right,
}

/// Represents information about a given row of notes in a chart
#[derive(Default, Debug)]
pub struct NoteRow {
    pub notes: [NoteType; 4],
    pub measure: u16,
    pub quantization: u8,
    /// Index of this row within its measure (`0..quantization`).
    ///
    /// Stored instead of the beat itself (1 byte vs 8); see [`NoteRow::beat`].
    pub slot: u8,
    pub seconds: f64,
}

impl NoteRow {
    /// Beat position of a row, derived from where it sits in its measure.
    pub fn beat_of(measure: u16, slot: u8, quantization: u8) -> f64 {
        (measure as f64 * 4.0) + 4.0 * slot as f64 / quantization as f64
    }

    /// Beat position of this row, counted from the start of the chart.
    pub fn beat(&self) -> f64 {
        Self::beat_of(self.measure, self.slot, self.quantization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_of_common_quantizations() {
        // 4th notes: one row per beat
        assert_eq!(NoteRow::beat_of(0, 1, 4), 1.0);
        // 8th notes
        assert_eq!(NoteRow::beat_of(0, 1, 8), 0.5);
        // 16th notes, second measure
        assert_eq!(NoteRow::beat_of(1, 3, 16), 4.75);
        // 192nd notes: one slot is 1/48 beat
        assert_eq!(NoteRow::beat_of(0, 1, 192), 4.0 / 192.0);
    }

    #[test]
    fn last_slot_lands_before_next_measure() {
        assert!(NoteRow::beat_of(2, 191, 192) < NoteRow::beat_of(3, 0, 192));
    }

    #[test]
    fn beat_method_matches_beat_of() {
        let row = NoteRow {
            measure: 5,
            quantization: 16,
            slot: 6,
            ..Default::default()
        };
        assert_eq!(row.beat(), NoteRow::beat_of(5, 6, 16));
    }
}
