#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use std::path::PathBuf;

fn create_test_file(name: &str, content: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("tva_test_join_header");
    fs::create_dir_all(&path).unwrap();
    path.push(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn join_multi_files_different_column_order() {
    // Filter file
    // Key: ID
    let filter_content = "ID\tVal
A\tFilterA
B\tFilterB";
    let filter_path = create_test_file("filter.tsv", filter_content);

    // File 1: ID is col 1
    let file1_content = "ID\tData
A\tDataA
C\tDataC";
    let file1_path = create_test_file("file1.tsv", file1_content);

    // File 2: ID is col 2 (swapped columns)
    let file2_content = "Data\tID
DataB\tB
DataD\tD";
    let file2_path = create_test_file("file2.tsv", file2_content);

    // Run join with -H and -k ID
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "--filter-file",
            filter_path.to_str().unwrap(),
            "--key-fields",
            "ID",
            "--append-fields",
            "Val",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    // Expected output:
    // Header (from file 1): ID\tData\tVal
    // Match A (file 1): A\tDataA\tFilterA
    // Match B (file 2): DataB\tB\tFilterB  (Note: output preserves file 2's column order, but appends correct val)

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "ID\tData\tVal");

    // Check file 1 match
    assert!(lines.contains(&"A\tDataA\tFilterA"));

    // Check file 2 match
    // For file 2, the line is "DataB\tB". Since key "B" matches filter, "FilterB" is appended.
    // So output line is "DataB\tB\tFilterB".
    assert!(lines.contains(&"DataB\tB\tFilterB"));
}
