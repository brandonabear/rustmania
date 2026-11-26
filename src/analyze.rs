use crate::song::{Chart, NoteRow, NoteType};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

pub fn analyze_chart(chart: &Chart) -> Result<ChartAnalysis, Error> {
    let analysis = ChartAnalysis {
        basic: calculate_basic_metrics(&chart.notes)?,
    };

    Ok(analysis)
}

pub fn calculate_basic_metrics(note_rows: &Vec<NoteRow>) -> Result<BasicMetrics, Error> {
    let mut basic = BasicMetrics::default();
    for row in note_rows {
        basic.update(row)?;
    }

    Ok(basic)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChartAnalysis {
    pub basic: BasicMetrics,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct BasicMetrics {
    pub steps: u64,
    pub taps: u64,
    pub jumps: u64,
    pub hands: u64,
    pub holds: u64,
    pub rolls: u64,
    pub mines: u64,
    pub lifts: u64,
    pub fakes: u64,
}

impl BasicMetrics {
    pub(crate) fn update(&mut self, note_row: &NoteRow) -> Result<()> {
        let mut heads: u64 = 0;
        for panel in note_row.notes {
            match panel {
                NoteType::Mine => self.mines += 1,
                NoteType::Fake => self.fakes += 1,
                NoteType::Lift => self.lifts += 1,
                NoteType::Roll => {
                    self.rolls += 1;
                    heads += 1
                }
                NoteType::Hold => {
                    self.holds += 1;
                    heads += 1
                }
                NoteType::Tap => heads += 1,
                _ => continue,
            }
        }

        self.steps += heads;
        if heads > 0 {
            self.taps += 1;
            if heads >= 2 {
                self.jumps += 1;
            }
            let covered_panels = (note_row.pressed() | note_row.held()).len();
            if covered_panels >= 3 {
                self.hands += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
