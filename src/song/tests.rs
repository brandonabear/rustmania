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
