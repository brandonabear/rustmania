use super::*;

/// Builds a `TimingEngine` from `(beat, value)` pairs.
fn engine(bpms: &[(f64, f64)], stops: &[(f64, f64)], offset: f64) -> TimingEngine {
    let to_changes = |pairs: &[(f64, f64)]| -> Vec<BeatChange> {
        pairs
            .iter()
            .map(|&(beat, value)| BeatChange { beat, value })
            .collect()
    };
    TimingEngine::new(&to_changes(bpms), &to_changes(stops), offset)
}

/// Floating-point comparison with a small tolerance.
#[track_caller]
fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

// Most tests use 120 BPM (0.5 s per beat) and 240 BPM (0.25 s per beat), which are exact in
// binary floating point.

#[test]
fn constant_bpm() {
    let t = engine(&[(0.0, 120.0)], &[], 0.0);
    assert_close(t.beats_to_seconds(0.0), 0.0);
    assert_close(t.beats_to_seconds(1.0), 0.5);
    assert_close(t.beats_to_seconds(3.5), 1.75);
    assert_close(t.beats_to_seconds(4.0), 2.0);
}

#[test]
fn offset_is_subtracted_from_every_beat() {
    let t = engine(&[(0.0, 120.0)], &[], 0.25);
    assert_close(t.beats_to_seconds(0.0), -0.25);
    assert_close(t.beats_to_seconds(4.0), 1.75);

    let t = engine(&[(0.0, 120.0)], &[], -0.1);
    assert_close(t.beats_to_seconds(4.0), 2.1);
}

#[test]
fn one_bpm_change() {
    let t = engine(&[(0.0, 120.0), (4.0, 240.0)], &[], 0.0);
    assert_close(t.beats_to_seconds(2.0), 1.0);
    // Exactly on the change: all time so far was at the old BPM
    assert_close(t.beats_to_seconds(4.0), 2.0);
    assert_close(t.beats_to_seconds(6.0), 2.5);
    assert_close(t.beats_to_seconds(8.0), 3.0);
}

#[test]
fn several_bpm_changes() {
    let t = engine(&[(0.0, 60.0), (2.0, 120.0), (4.0, 240.0)], &[], 0.0);
    assert_close(t.beats_to_seconds(2.0), 2.0);
    assert_close(t.beats_to_seconds(3.0), 2.5);
    assert_close(t.beats_to_seconds(4.0), 3.0);
    assert_close(t.beats_to_seconds(5.0), 3.25);
}

#[test]
fn last_bpm_segment_is_open_ended() {
    let t = engine(&[(0.0, 120.0), (4.0, 240.0)], &[], 0.0);
    assert_close(t.beats_to_seconds(100.0), 2.0 + 96.0 * 0.25);
}

#[test]
fn stop_before_target_adds_its_pause() {
    let t = engine(&[(0.0, 120.0)], &[(2.0, 1.5)], 0.0);
    assert_close(t.beats_to_seconds(1.0), 0.5);
    assert_close(t.beats_to_seconds(3.0), 1.5 + 1.5);
}

#[test]
fn stop_at_target_beat_is_not_counted() {
    // A note on the same beat as a stop is hit when the stop starts, not when it ends.
    // ITGMania orders it the same way: TimingData.cpp checks the target beat before stops.
    let t = engine(&[(0.0, 120.0)], &[(2.0, 1.5)], 0.0);
    assert_close(t.beats_to_seconds(2.0), 1.0);
}

#[test]
fn several_stops_accumulate() {
    let t = engine(&[(0.0, 120.0)], &[(1.0, 0.5), (3.0, 0.25)], 0.0);
    assert_close(t.beats_to_seconds(2.0), 1.0 + 0.5);
    assert_close(t.beats_to_seconds(4.0), 2.0 + 0.5 + 0.25);
}

#[test]
fn stop_on_a_bpm_change() {
    let t = engine(&[(0.0, 120.0), (4.0, 60.0)], &[(4.0, 1.0)], 0.0);
    assert_close(t.beats_to_seconds(4.0), 2.0);
    // 4 beats at 120 BPM, the 1 s stop, then 2 beats at 60 BPM
    assert_close(t.beats_to_seconds(6.0), 2.0 + 1.0 + 2.0);
}

#[test]
fn bpms_stops_and_offset_combined() {
    let t = engine(
        &[(0.0, 120.0), (4.0, 240.0)],
        &[(2.0, 0.5), (6.0, 0.25)],
        0.1,
    );
    // 4 beats at 120 + 4 beats at 240 + both stops - offset
    assert_close(t.beats_to_seconds(8.0), 2.0 + 1.0 + 0.5 + 0.25 - 0.1);
}

#[test]
fn unsorted_input_is_sorted() {
    let sorted = engine(
        &[(0.0, 120.0), (4.0, 240.0)],
        &[(1.0, 0.5), (3.0, 0.25)],
        0.0,
    );
    let unsorted = engine(
        &[(4.0, 240.0), (0.0, 120.0)],
        &[(3.0, 0.25), (1.0, 0.5)],
        0.0,
    );
    for beat in [0.0, 1.0, 2.0, 3.5, 4.0, 6.0, 10.0] {
        assert_close(
            unsorted.beats_to_seconds(beat),
            sorted.beats_to_seconds(beat),
        );
    }
}
