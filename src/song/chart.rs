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

/// Represents the chart-level metadata of a simfile
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChartMetadata {
    pub mode: Option<String>,
    pub description: Option<String>,
    pub credit: Option<String>,
    pub chart_name: Option<String>,
    pub difficulty: Option<Difficulty>,
    pub rating: Option<f64>,
    pub radar: Option<GrooveRadar>,
}

impl ChartMetadata {
    pub(crate) fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // Return None for empty values
        let value = value.trim();
        if value.is_empty() {
            return Ok(());
        }

        match key {
            "STEPSTYPE" => self.mode = Some(value.to_owned()),
            "DESCRIPTION" => self.description = Some(value.to_owned()),
            "CREDIT" => self.credit = Some(value.to_owned()),
            "CHARTNAME" => self.chart_name = Some(value.to_owned()),
            "DIFFICULTY" => self.difficulty = Some(value.parse::<Difficulty>()?),
            "METER" => self.rating = Some(value.parse::<f64>()?),
            "RADARVALUES" => self.radar = Some(value.parse::<GrooveRadar>()?),
            _ => {}
        }
        Ok(())
    }
}

/// Enum of valid chart Difficulties
#[derive(Serialize, Deserialize, Debug)]
pub enum Difficulty {
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
    pub stream: f64,
    pub voltage: f64,
    pub air: f64,
    pub freeze: f64,
    pub chaos: f64,
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
            stream: values[0].parse::<f64>()?,
            voltage: values[1].parse::<f64>()?,
            air: values[2].parse::<f64>()?,
            freeze: values[3].parse::<f64>()?,
            chaos: values[4].parse::<f64>()?,
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
