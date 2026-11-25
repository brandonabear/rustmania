use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Represents the song-level metadata of a simfile
#[derive(Serialize, Deserialize, Debug, Default)]
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
    /// Sets the field matching a `.ssc` header tag.
    pub(crate) fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // Return None for empty values
        let value = value.trim();
        if value.is_empty() {
            return Ok(());
        }
        
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

/// Represents the beat at which a time change is applied.
///
/// A `BeatChange` usually represents BPM changes or Stops.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BeatChange {
    pub beat: f64,
    pub value: f64,
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
