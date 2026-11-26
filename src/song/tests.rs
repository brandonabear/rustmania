use super::*;

#[test]
fn beat_of_common_quantizations() {
    // 4th notes: one row per beat
    assert_eq!(NoteRow::beat_of(0, 1, 4), 1.0);
    // 8th notes
    assert_eq!(NoteRow::beat_of(0, 1, 8), 0.5);
    // 16th notes, second measure
    assert_eq!(NoteRow::beat_of(1, 3, 16), 4.75);
    // 192nd notes: one slot is 1/48 beat
    assert_eq!(NoteRow::beat_of(0, 1, 192), 4.0 / 192.0);
}

#[test]
fn last_slot_lands_before_next_measure() {
    assert!(NoteRow::beat_of(2, 191, 192) < NoteRow::beat_of(3, 0, 192));
}

#[test]
fn beat_method_matches_beat_of() {
    let row = NoteRow {
        measure: 5,
        quantization: 16,
        slot: 6,
        ..Default::default()
    };
    assert_eq!(row.beat(), NoteRow::beat_of(5, 6, 16));
}

// ---------------------------------------------------------------------------
// GrooveRadar::from_str
// ---------------------------------------------------------------------------

/// Radar values `1,2,...,n`. Whole numbers, so they also parse as counts
fn radar_values(n: usize) -> String {
    (1..=n).map(|i| i.to_string()).collect::<Vec<_>>().join(",")
}

fn counts(r: &GrooveRadar) -> [Option<u32>; 9] {
    [
        r.steps, r.taps, r.jumps, r.holds, r.mines, r.hands, r.rolls, r.lifts, r.fakes,
    ]
}

#[test]
fn radar_five_values_has_no_counts() {
    let r: GrooveRadar = "0.1,0.2,0.3,0.4,0.5".parse().unwrap();
    assert_eq!(
        [r.stream, r.voltage, r.air, r.freeze, r.chaos],
        [0.1, 0.2, 0.3, 0.4, 0.5]
    );
    assert_eq!(counts(&r), [None; 9]);
}

#[test]
fn radar_eleven_values_has_counts_up_to_hands() {
    let r: GrooveRadar = radar_values(11).parse().unwrap();
    assert_eq!(r.chaos, 5.0);
    assert_eq!(
        counts(&r),
        [
            Some(6),
            Some(7),
            Some(8),
            Some(9),
            Some(10),
            Some(11),
            None,
            None,
            None
        ]
    );
}

#[test]
fn radar_fourteen_values_has_all_counts() {
    let r: GrooveRadar = radar_values(14).parse().unwrap();
    assert_eq!(counts(&r), (6..=14).map(Some).collect::<Vec<_>>()[..]);
}

#[test]
fn radar_twenty_two_values_reads_first_player_only() {
    // Second player's block is 12..=22; none of it should appear
    let r: GrooveRadar = radar_values(22).parse().unwrap();
    assert_eq!(r.stream, 1.0);
    assert_eq!(r.hands, Some(11));
    assert_eq!(r.rolls, None);
}

#[test]
fn radar_twenty_eight_values_reads_first_player_only() {
    let r: GrooveRadar = radar_values(28).parse().unwrap();
    assert_eq!(r.stream, 1.0);
    assert_eq!(r.fakes, Some(14));
}

#[test]
fn radar_tolerates_whitespace() {
    let r: GrooveRadar = " 0.1 , 0.2,0.3 ,0.4,  0.5 ".parse().unwrap();
    assert_eq!(r.stream, 0.1);
    assert_eq!(r.chaos, 0.5);
}

#[test]
fn radar_error_on_too_few_values() {
    let err = "0.1,0.2,0.3,0.4".parse::<GrooveRadar>().unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected at least 5 values, got 4"),
        "{err}"
    );
}

#[test]
fn radar_error_on_non_numeric_value() {
    assert!("0.1,0.2,abc,0.4,0.5".parse::<GrooveRadar>().is_err());
}

#[test]
fn radar_error_on_fractional_count() {
    // Counts are whole numbers
    assert!("0.1,0.2,0.3,0.4,0.5,1.5".parse::<GrooveRadar>().is_err());
}

// ---------------------------------------------------------------------------
// AnchorTracker
// ---------------------------------------------------------------------------

/// The four notes of a row as written in a simfile, e.g. `"2M00"`
fn cols(row: &str) -> anchor::Columns {
    let chars: Vec<char> = row.chars().collect();
    assert_eq!(chars.len(), 4, "row must have 4 columns: {row:?}");
    std::array::from_fn(|i| NoteType::try_from(chars[i]).unwrap())
}

/// Feed rows to a fresh tracker in order, collecting `(holds, rolls)` for each
fn track(rows: &[&str]) -> Vec<(Panel, Panel)> {
    let mut tracker = AnchorTracker::default();
    rows.iter().map(|row| tracker.advance(cols(row))).collect()
}

const NONE: Panel = Panel::empty();

#[test]
fn nothing_active_without_anchors() {
    assert_eq!(
        track(&["0000", "1000", "0M01", "1111"]),
        vec![(NONE, NONE); 4]
    );
}

#[test]
fn hold_active_from_head_through_tail() {
    assert_eq!(
        track(&["2000", "0100", "3000", "1000"]),
        vec![
            (Panel::LEFT, NONE), // head row
            (Panel::LEFT, NONE), // underway
            (Panel::LEFT, NONE), // tail row is still active
            (NONE, NONE),        // released
        ]
    );
}

#[test]
fn roll_tracked_as_roll_not_hold() {
    assert_eq!(
        track(&["0004", "0000", "0003", "0001"]),
        vec![
            (NONE, Panel::RIGHT),
            (NONE, Panel::RIGHT),
            (NONE, Panel::RIGHT),
            (NONE, NONE),
        ]
    );
}

#[test]
fn head_and_tail_on_same_row_in_different_columns() {
    // Left's hold ends on the same row that Up's hold starts
    assert_eq!(
        track(&["2000", "3020", "0030"]),
        vec![
            (Panel::LEFT, NONE),
            (Panel::LEFT | Panel::UP, NONE),
            (Panel::UP, NONE),
        ]
    );
}

#[test]
fn overlapping_hold_and_roll_end_independently() {
    assert_eq!(
        track(&["2004", "0110", "3000", "0003", "0000"]),
        vec![
            (Panel::LEFT, Panel::RIGHT),
            (Panel::LEFT, Panel::RIGHT),
            (Panel::LEFT, Panel::RIGHT),
            (NONE, Panel::RIGHT),
            (NONE, NONE),
        ]
    );
}

#[test]
fn simultaneous_holds_on_every_panel() {
    assert_eq!(
        track(&["2222", "3333", "0000"]),
        vec![(Panel::all(), NONE), (Panel::all(), NONE), (NONE, NONE)]
    );
}

#[test]
fn column_can_be_reused_after_tail() {
    assert_eq!(
        track(&["0200", "0300", "0400", "0300"]),
        vec![
            (Panel::DOWN, NONE),
            (Panel::DOWN, NONE),
            (NONE, Panel::DOWN),
            (NONE, Panel::DOWN),
        ]
    );
}

#[test]
fn stray_tail_is_ignored() {
    // A tail with no head (malformed chart) shouldn't panic or activate anything
    assert_eq!(track(&["3000", "1000"]), vec![(NONE, NONE); 2]);
}

#[test]
fn other_notes_do_not_affect_active_anchors() {
    // Taps, mines, fakes and lifts in other columns leave the hold alone
    assert_eq!(
        track(&["2000", "01M0", "0FL0", "3000"]),
        vec![(Panel::LEFT, NONE); 4]
    );
}
