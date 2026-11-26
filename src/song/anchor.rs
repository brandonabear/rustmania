use crate::song::{NoteType, PANELS, Panel};
use std::iter::zip;

pub type Columns = [NoteType; 4];

#[derive(Clone, Copy, Eq, PartialEq, Default, Debug)]
pub enum AnchorType {
    #[default]
    Idle,
    Hold,
    Roll,
}

/// Track anchored panels with active holds/rolls
#[derive(Debug, Default)]
pub struct AnchorTracker {
    state: [AnchorType; 4],
}

impl AnchorTracker {
    /// Update anchored state for a given row of notes
    ///
    /// State is still active through the tail row inclusive
    pub(crate) fn advance(&mut self, notes: Columns) -> (Panel, Panel) {
        self.apply_heads(notes);
        let active = self.snapshot();
        self.apply_tails(notes);

        active
    }

    fn apply_heads(&mut self, notes: Columns) {
        for (col, note) in notes.iter().enumerate() {
            match note {
                NoteType::Hold => self.state[col] = AnchorType::Hold,
                NoteType::Roll => self.state[col] = AnchorType::Roll,
                _ => (),
            }
        }
    }

    fn apply_tails(&mut self, notes: Columns) {
        for (col, note) in notes.iter().enumerate() {
            if note == &NoteType::Tail {
                self.state[col] = AnchorType::Idle
            }
        }
    }

    fn snapshot(&mut self) -> (Panel, Panel) {
        let mut holds = Panel::empty();
        let mut rolls = Panel::empty();

        for (panel, state) in zip(PANELS, self.state) {
            match state {
                AnchorType::Hold => holds |= panel,
                AnchorType::Roll => rolls |= panel,
                _ => (),
            }
        }
        (holds, rolls)
    }
}
