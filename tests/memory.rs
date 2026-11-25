//! Heap usage measurement for `parse_simfile`.
//!
//! dhat's stats are process-global, so this file must contain exactly one test.
//! Run with `cargo test --test memory -- --nocapture` to see the numbers.

use rustmania::parse::parse;
use std::fs;
use std::path::PathBuf;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn parse_simfile_heap_usage() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let path = PathBuf::from("data/fots.ssc");
    let content = fs::read_to_string(&path).unwrap();
    let file_bytes = content.len();

    let before = dhat::HeapStats::get();
    let song = parse(&path).unwrap();
    let after = dhat::HeapStats::get();

    println!("file size:        {file_bytes} bytes");
    println!("peak (max_bytes): {} bytes", after.max_bytes);
    println!("live at end:      {} bytes", after.curr_bytes);
    println!("live before parse:{} bytes", before.curr_bytes);
    println!(
        "allocated during parse: {} bytes in {} blocks",
        after.total_bytes - before.total_bytes,
        after.total_blocks - before.total_blocks
    );

    // Keep `song` alive until after the final stats are taken.
    assert!(!song.charts.is_empty());
}
