use anyhow::{Result, bail};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::iter::zip;

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

impl NoteType {
    pub fn is_pressable(self) -> bool {
        matches!(self, Self::Tap | Self::Hold | Self::Roll)
    }

    pub fn is_tail(self) -> bool {
        self == Self::Tail
    }

    pub fn is_mine(self) -> bool {
        self == Self::Mine
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
    pub struct Panel: u8 {
        const LEFT  = 0b0001;
        const DOWN  = 0b0010;
        const UP    = 0b0100;
        const RIGHT = 0b1000;
    }
}

impl Panel {
    pub fn column(self) -> Option<u8> {
        if self.bits().count_ones() == 1 {
            Some(self.bits().trailing_zeros() as u8)
        } else {
            None
        }
    }

    pub fn single(self) -> Self {
        if self.bits().count_ones() == 1 {
            self
        } else {
            Self::empty()
        }
    }

    pub fn len(&self) -> u32 {
        self.bits().count_ones()
    }
}

pub const PANELS: [Panel; 4] = [Panel::LEFT, Panel::DOWN, Panel::UP, Panel::RIGHT];

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
    pub holds_active: Panel,
    pub rolls_active: Panel,
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

    pub fn panels_with(&self, kinds: impl Fn(NoteType) -> bool) -> Panel {
        let mut found = Panel::empty();
        for (panel, note) in zip(PANELS, self.notes.iter()) {
            if kinds(*note) {
                found |= panel
            }
        }
        found
    }

    pub fn mines(&self) -> Panel {
        self.panels_with(NoteType::is_mine)
    }

    pub fn pressed(&self) -> Panel {
        self.panels_with(NoteType::is_pressable)
    }

    pub fn held(&self) -> Panel {
        (self.holds_active | self.rolls_active) - self.pressed()
    }
}
