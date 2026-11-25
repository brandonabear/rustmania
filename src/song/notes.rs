use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// Represents all possible panel values
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Default, Debug)]
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

// /// Represents all possible directions (panels)
// #[derive(Debug, Clone, Copy)]
// pub enum Column {
//     Left,
//     Down,
//     Up,
//     Right,
// }

/// Represents information about a given row of notes in a chart
#[derive(Serialize, Deserialize, Default, Debug)]
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
    pub(crate) fn beat_of(measure: u16, slot: u8, quantization: u8) -> f64 {
        (measure as f64 * 4.0) + 4.0 * slot as f64 / quantization as f64
    }

    /// Beat position of this row, counted from the start of the chart.
    pub fn beat(&self) -> f64 {
        Self::beat_of(self.measure, self.slot, self.quantization)
    }
}
