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
