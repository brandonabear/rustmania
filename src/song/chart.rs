use crate::song::NoteRow;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a given chart within a song
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Chart {
    pub metadata: ChartMetadata,
    pub notes: Vec<NoteRow>,
}

/// Enum of valid chart Difficulties
#[derive(Serialize, Deserialize, Default, Debug)]
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
        match s.to_ascii_lowercase().as_str() {
            "beginner" => Ok(Difficulty::Beginner),
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            "challenge" => Ok(Difficulty::Challenge),
            "edit" => Ok(Difficulty::Edit),
            other => bail!("Unknown Difficulty: {other}"),
        }
    }
}

/// Represents statistics that drive the GrooveRadar game element
#[derive(Serialize, Deserialize, Debug, Default)]
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
#[derive(Serialize, Deserialize, Debug, Default)]
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
    pub(crate) fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let value = value.trim();
        match key {
            "STEPSTYPE" => self.mode = value.to_owned(),
            "DESCRIPTION" => self.description = value.to_owned(),
            "CREDIT" => self.credit = value.to_owned(),
            "CHARTNAME" => self.chart_name = value.to_owned(),
            "DIFFICULTY" => self.difficulty = value.parse::<Difficulty>()?,
            "METER" => self.rating = value.parse::<f64>()?,
            "RADARVALUES" => self.radar = value.parse::<GrooveRadar>()?,
            _ => {}
        }
        Ok(())
    }
}
