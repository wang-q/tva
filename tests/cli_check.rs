#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn check_valid_ctg() -> anyhow::Result<()> {
    let (stdout, _) = TvaCmd::new().args(&["check", "tests/genome/ctg.tsv"]).run();

    assert!(stdout.contains("4 lines, 6 fields"));

    Ok(())
}

#[test]
fn check_empty_input() -> anyhow::Result<()> {
    let (stdout, _) = TvaCmd::new().stdin("").args(&["check"]).run();

    assert!(stdout.contains("0 lines, 0 fields"));

    Ok(())
}

#[test]
fn check_simple_matrix() -> anyhow::Result<()> {
    let input = "A\t1\t!\nB\t2\t@\nC\t3\t#\nD\t4\t$\nE\t5\t%\n";

    let (stdout, _) = TvaCmd::new().stdin(input).args(&["check"]).run();

    assert!(stdout.contains("5 lines, 3 fields"));

    Ok(())
}

#[test]
fn check_invalid_structure_from_stdin() -> anyhow::Result<()> {
    let input = "a\tb\tc\n1\t2\n";

    let (_, stderr) = TvaCmd::new().stdin(input).args(&["check"]).run_fail();

    assert!(stderr.contains("line 2 (2 fields):"));
    assert!(stderr.contains(
        "tva check: structure check failed: line 2 has 2 fields (expected 3)"
    ));

    Ok(())
}

#[test]
fn check_empty_line_zero_fields() -> anyhow::Result<()> {
    let input = "x\ty\n\nu\tv\n";

    let (_, stderr) = TvaCmd::new().stdin(input).args(&["check"]).run_fail();

    assert!(stderr.contains("line 2 (0 fields):"));
    assert!(stderr.contains(
        "tva check: structure check failed: line 2 has 0 fields (expected 2)"
    ));

    Ok(())
}

#[test]
fn check_multiple_files_fail_second() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "a\tb\n1\t2\n")?;
    fs::write(&file2, "a\tb\n1\t2\t3\n")?;

    let file1_str = file1.to_str().unwrap();
    let file2_str = file2.to_str().unwrap();

    let (_, stderr) = TvaCmd::new()
        .args(&["check", file1_str, file2_str])
        .run_fail();

    assert!(stderr.contains("structure check failed"));

    Ok(())
}

#[test]
fn check_file_open_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["check", "non_existent_file_check.tsv"])
        .run_fail();

    assert!(stderr.contains("could not open"));
}

#[test]
fn check_lossy_utf8_error_display() {
    // Tests L41: eprintln!("  {}", String::from_utf8_lossy(record));

    // First line has 3 fields.
    // Second line has 2 fields (with invalid UTF-8).
    // We need second line to have DIFFERENT number of fields to trigger error printing.
    let input = b"col1\tcol2\tcol3\n\xFF\tval2\n"; // \xFF is invalid UTF-8

    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("invalid_utf8.tsv");
    fs::write(&file_path, input).unwrap();

    let (_, stderr) = TvaCmd::new()
        .args(&["check", file_path.to_str().unwrap()])
        .run_fail();

    // Should contain replacement character for \xFF
    assert!(stderr.contains("\u{FFFD}"));
    assert!(stderr.contains("line 2 (2 fields)"));
    assert!(stderr.contains("expected 3"));
}

#[test]
fn check_multiple_files_consistent() {
    // Test multiple files that ARE consistent (L25 loop continuation)
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "a\tb\n1\t2\n").unwrap();
    fs::write(&file2, "3\t4\n5\t6\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["check", file1.to_str().unwrap(), file2.to_str().unwrap()])
        .run();

    assert!(stdout.contains("4 lines, 2 fields"));
}

#[test]
fn check_with_header_firstline() {
    // Test --header (FirstLine mode)
    let input = "col1\tcol2\tcol3\n1\t2\t3\n4\t5\t6\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["check", "--header"])
        .run();

    // Should count header line (1) + data lines (2) = 3 total
    assert!(stdout.contains("3 lines total, 2 data lines, 3 fields"));
}

#[test]
fn check_with_header_lines_n() {
    // Test --header-lines N (LinesN mode)
    let input = "# comment\ncol1\tcol2\n1\t2\n3\t4\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["check", "--header-lines", "2"])
        .run();

    // Should count 2 header lines and report fields from first data row
    assert!(stdout.contains("4 lines total, 2 data lines"));
}

#[test]
fn check_with_header_hash() {
    // Test --header-hash (HashLines mode)
    let input = "# comment 1\n# comment 2\n1\t2\t3\n4\t5\t6\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["check", "--header-hash"])
        .run();

    // Should count hash lines and report fields from first data row
    assert!(stdout.contains("4 lines total, 2 data lines"));
}

#[test]
fn check_with_header_hash1() {
    // Test --header-hash1 (HashLines1 mode)
    let input = "# comment\ncol1\tcol2\tcol3\n1\t2\t3\n4\t5\t6\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["check", "--header-hash1"])
        .run();

    // Should count hash line + column names line
    assert!(stdout.contains("4 lines total, 2 data lines, 3 fields"));
}

#[test]
fn check_empty_file_with_header() {
    // Test empty file with --header (header_result is None)
    let (stdout, _) = TvaCmd::new().stdin("").args(&["check", "--header"]).run();

    assert!(stdout.contains("0 lines, 0 fields"));
}

#[test]
fn check_multiple_files_with_header() {
    // Test multiple files with header mode
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "col1\tcol2\n1\t2\n").unwrap();
    fs::write(&file2, "col1\tcol2\n3\t4\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "check",
            "--header",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    // Should count headers from both files
    assert!(stdout.contains("4 lines total, 2 data lines, 2 fields"));
}

#[test]
fn check_header_field_mismatch() {
    // Test header with different field count than data
    let input = "col1\tcol2\tcol3\n1\t2\n";

    let (_, stderr) = TvaCmd::new()
        .stdin(input)
        .args(&["check", "--header"])
        .run_fail();

    assert!(stderr.contains("line 2 (2 fields)"));
    assert!(stderr.contains("expected 3"));
}

#[test]
fn check_header_empty_column_names() {
    // Test header with empty column names line (edge case)
    // Empty header line means 0 fields expected, data has 2 fields
    // This should fail because field count doesn't match
    let input = "\n1\t2\n";

    let (_, stderr) = TvaCmd::new()
        .stdin(input)
        .args(&["check", "--header"])
        .run_fail();

    // Should report error: line 2 has 2 fields but expected 0
    assert!(stderr.contains("line 2 (2 fields)"));
    assert!(stderr.contains("expected 0"));
}
