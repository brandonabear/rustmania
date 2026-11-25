use super::*;
use crate::song::BeatChange;

// ---------------------------------------------------------------------------
// strip_comments
// ---------------------------------------------------------------------------

#[test]
fn strip_comments_removes_trailing_comment() {
    assert_eq!(strip_comments("#TITLE:Foo; // a comment"), "#TITLE:Foo; ");
}

#[test]
fn strip_comments_removes_whole_line_comment() {
    assert_eq!(strip_comments("// header\n#TITLE:Foo;"), "\n#TITLE:Foo;");
}

#[test]
fn strip_comments_leaves_lines_without_comments_alone() {
    let text = "#TITLE:Foo;\n#ARTIST:Bar;";
    assert_eq!(strip_comments(text), text);
}

#[test]
fn strip_comments_handles_each_line_separately() {
    let text = "0000 // measure 1\n1000\n,  // measure 2\n0100";
    assert_eq!(strip_comments(text), "0000 \n1000\n,  \n0100");
}

#[test]
fn strip_comments_normalizes_line_endings() {
    // `str::lines` accepts both `\n` and `\r\n`, and the output is rejoined with `\n`
    assert_eq!(strip_comments("a // x\r\nb\r\n"), "a \nb");
}

// ---------------------------------------------------------------------------
// get_key_values
// ---------------------------------------------------------------------------

fn kv(s: &str) -> Vec<(&str, &str)> {
    get_key_values(s).collect()
}

#[test]
fn key_values_basic() {
    assert_eq!(
        kv("#TITLE:Foo;#ARTIST:Bar;"),
        [("TITLE", "Foo"), ("ARTIST", "Bar")]
    );
}

#[test]
fn key_values_ignore_whitespace_between_tags() {
    assert_eq!(
        kv("\n#TITLE:Foo;\n  \t#ARTIST:Bar;\n"),
        [("TITLE", "Foo"), ("ARTIST", "Bar")]
    );
}

#[test]
fn key_values_split_on_first_colon_only() {
    assert_eq!(kv("#TITLE:Re:Zero;"), [("TITLE", "Re:Zero")]);
}

#[test]
fn key_values_keep_empty_values() {
    // `Header::set` and `ChartMetadata::set` turn these into `None`
    assert_eq!(kv("#GENRE:;"), [("GENRE", "")]);
}

#[test]
fn key_values_leave_leading_whitespace_for_set_to_trim() {
    assert_eq!(kv("#TITLE:  Foo  ;"), [("TITLE", "  Foo")]);
}

#[test]
fn key_values_keep_multiline_values() {
    assert_eq!(
        kv("#NOTES:\n0000\n1000\n,\n0100\n;"),
        [("NOTES", "\n0000\n1000\n,\n0100")]
    );
}

#[test]
fn key_values_skip_chunks_without_a_colon() {
    assert_eq!(kv("garbage;#TITLE:Foo;"), [("TITLE", "Foo")]);
}

#[test]
fn key_values_accept_missing_hash() {
    assert_eq!(kv("TITLE:Foo;"), [("TITLE", "Foo")]);
}

#[test]
fn key_values_after_strip_comments() {
    let text = strip_comments("#TITLE:Foo; // song title\n#BPMS:0=120; // tempo");
    assert_eq!(kv(&text), [("TITLE", "Foo"), ("BPMS", "0=120")]);
}

// ---------------------------------------------------------------------------
// parse_notes
// ---------------------------------------------------------------------------

/// 120 BPM, no stops, no offset: one beat is 0.5 s
fn timing() -> TimingEngine {
    TimingEngine::new(
        &[BeatChange {
            beat: 0.0,
            value: 120.0,
        }],
        &[],
        0.0,
    )
}

fn notes(text: &str) -> Result<Chart> {
    let mut chart = Chart::default();
    parse_notes(&timing(), &mut chart, text)?;
    Ok(chart)
}

/// The full error message, including every `.context()` layer
fn error_message(text: &str) -> String {
    format!("{:#}", notes(text).unwrap_err())
}

#[test]
fn notes_skip_empty_rows_but_keep_their_slot() {
    let chart = notes("\n0000\n1000\n0000\n0001\n").unwrap();
    assert_eq!(chart.notes.len(), 2);

    let first = &chart.notes[0];
    assert_eq!(
        first.notes,
        [
            NoteType::Tap,
            NoteType::Empty,
            NoteType::Empty,
            NoteType::Empty
        ]
    );
    assert_eq!((first.measure, first.quantization, first.slot), (0, 4, 1));
    assert_eq!(first.seconds, 0.5);

    let second = &chart.notes[1];
    assert_eq!(
        (second.measure, second.quantization, second.slot),
        (0, 4, 3)
    );
    assert_eq!(second.seconds, 1.5);
}

#[test]
fn notes_quantization_comes_from_rows_per_measure() {
    let chart = notes("1000\n0000\n0000\n0000\n,\n1000\n0000\n").unwrap();
    assert_eq!(chart.notes.len(), 2);
    assert_eq!(
        (chart.notes[0].measure, chart.notes[0].quantization),
        (0, 4)
    );
    assert_eq!(
        (chart.notes[1].measure, chart.notes[1].quantization),
        (1, 2)
    );
    // Second measure starts at beat 4 = 2.0 s
    assert_eq!(chart.notes[1].seconds, 2.0);
}

#[test]
fn notes_ignore_surrounding_whitespace() {
    let chart = notes("\n  1000  \n\t0000\n\n").unwrap();
    assert_eq!(chart.notes.len(), 1);
    assert_eq!(chart.notes[0].quantization, 2);
}

#[test]
fn notes_error_on_too_many_columns() {
    let msg = error_message("0000\n00000\n");
    assert!(msg.contains("Expected 4 columns, got 5"), "{msg}");
    assert!(msg.contains("Measure 0"), "{msg}");
}

#[test]
fn notes_error_on_too_few_columns() {
    let msg = error_message("0000\n,\n000\n");
    assert!(msg.contains("Expected 4 columns, got 3"), "{msg}");
    assert!(msg.contains("Measure 1"), "{msg}");
}

#[test]
fn notes_error_on_unknown_character() {
    let msg = error_message("00X0\n");
    assert!(msg.contains("Unknown note character: 'X'"), "{msg}");
}

#[test]
fn notes_error_on_non_ascii_character() {
    // Counted in chars, not bytes, so this is reported as a bad character, not a bad width
    let msg = error_message("00é0\n");
    assert!(msg.contains("Unknown note character: 'é'"), "{msg}");
}

#[test]
fn notes_allow_255_rows_per_measure() {
    let text = "1000\n".repeat(255);
    let chart = notes(&text).unwrap();
    assert_eq!(chart.notes.len(), 255);
    assert_eq!(chart.notes[0].quantization, 255);
}

#[test]
fn notes_error_on_more_than_255_rows_per_measure() {
    let text = "0000\n".repeat(256);
    let msg = error_message(&text);
    assert!(msg.contains("Measure 0 has more than 255 rows"), "{msg}");
}
