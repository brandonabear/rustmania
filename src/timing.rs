use crate::song::BeatChange;

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
