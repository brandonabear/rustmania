use super::*;
use crate::song::AnchorTracker;

/// Build note rows from simfile row text (e.g. `"2M00"`), with hold/roll state
/// filled in by `AnchorTracker` the same way the parser does
fn rows(text: &[&str]) -> Vec<NoteRow> {
    let mut tracker = AnchorTracker::default();
    text.iter()
        .map(|row| {
            let chars: Vec<char> = row.chars().collect();
            assert_eq!(chars.len(), 4, "row must have 4 columns: {row:?}");
            let notes = std::array::from_fn(|i| NoteType::try_from(chars[i]).unwrap());
            let (holds_active, rolls_active) = tracker.advance(notes);
            NoteRow {
                notes,
                holds_active,
                rolls_active,
                ..Default::default()
            }
        })
        .collect()
}

fn metrics(text: &[&str]) -> BasicMetrics {
    calculate_basic_metrics(&rows(text)).unwrap()
}

// ---------------------------------------------------------------------------
// Counting notes
// ---------------------------------------------------------------------------

#[test]
fn empty_chart_has_no_counts() {
    assert_eq!(metrics(&[]), BasicMetrics::default());
}

#[test]
fn single_taps() {
    assert_eq!(
        metrics(&["1000", "0100", "0010", "0001"]),
        BasicMetrics {
            steps: 4,
            taps: 4,
            ..Default::default()
        }
    );
}

#[test]
fn jump_is_one_tap_row_and_two_steps() {
    assert_eq!(
        metrics(&["1001"]),
        BasicMetrics {
            steps: 2,
            taps: 1,
            jumps: 1,
            ..Default::default()
        }
    );
}

#[test]
fn hold_and_roll_heads_count_as_steps() {
    assert_eq!(
        metrics(&["2004", "3003"]),
        BasicMetrics {
            steps: 2,
            taps: 1,
            jumps: 1,
            holds: 1,
            rolls: 1,
            ..Default::default()
        }
    );
}

#[test]
fn tails_alone_count_nothing() {
    let m = metrics(&["2000", "3000"]);
    assert_eq!((m.steps, m.taps, m.holds), (1, 1, 1));
}

#[test]
fn mines_fakes_and_lifts_are_counted_but_not_stepped() {
    assert_eq!(
        metrics(&["MFL0", "M000"]),
        BasicMetrics {
            mines: 2,
            fakes: 1,
            lifts: 1,
            ..Default::default()
        }
    );
}

#[test]
fn mine_beside_tap_does_not_make_a_jump() {
    assert_eq!(
        metrics(&["1M00"]),
        BasicMetrics {
            steps: 1,
            taps: 1,
            mines: 1,
            ..Default::default()
        }
    );
}

// ---------------------------------------------------------------------------
// Hands: 3+ panels pressed or held on a row that has at least one press
// ---------------------------------------------------------------------------

#[test]
fn three_taps_is_a_hand_and_a_jump() {
    let m = metrics(&["1110"]);
    assert_eq!((m.steps, m.taps, m.jumps, m.hands), (3, 1, 1, 1));
}

#[test]
fn quad_is_a_hand_and_a_jump() {
    let m = metrics(&["1111"]);
    assert_eq!((m.steps, m.taps, m.jumps, m.hands), (4, 1, 1, 1));
}

#[test]
fn jump_while_holding_is_a_hand() {
    // Left is held from the first row, so the Down+Up jump covers 3 panels
    let m = metrics(&["2000", "0110", "3000"]);
    assert_eq!((m.jumps, m.hands), (1, 1));
}

#[test]
fn tap_while_holding_two_is_a_hand_but_not_a_jump() {
    let m = metrics(&["2200", "0010", "3300"]);
    assert_eq!((m.jumps, m.hands), (1, 1)); // the jump is the "2200" row
    let m = metrics(&["2000", "0200", "0010", "3300"]);
    assert_eq!((m.jumps, m.hands), (0, 1));
}

#[test]
fn rolls_count_toward_hands_like_holds() {
    let m = metrics(&["4000", "0110", "3000"]);
    assert_eq!(m.hands, 1);
}

#[test]
fn held_panels_alone_do_not_make_a_hand() {
    // Three panels held, but the mine and tail rows press nothing new
    let m = metrics(&["2220", "000M", "3330"]);
    assert_eq!(m.hands, 1); // only the head row
}

#[test]
fn hand_only_counted_while_holds_are_active() {
    // After Left's tail, the Down+Up jump only covers 2 panels
    let m = metrics(&["2000", "3000", "0110"]);
    assert_eq!((m.jumps, m.hands), (1, 0));
}

#[test]
fn tap_on_tail_row_still_sees_the_hold() {
    // A hold is active through its tail row, so this jump covers 3 panels
    let m = metrics(&["2000", "3110"]);
    assert_eq!((m.jumps, m.hands), (1, 1));
}

// ---------------------------------------------------------------------------
// analyze_chart
// ---------------------------------------------------------------------------

#[test]
fn analyze_chart_uses_chart_notes() {
    let chart = Chart {
        notes: rows(&["1000", "1100"]),
        ..Default::default()
    };
    let analysis = analyze_chart(&chart).unwrap();
    assert_eq!(analysis.basic, metrics(&["1000", "1100"]));
}
