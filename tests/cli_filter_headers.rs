#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

// ============================================================================
// Tests for --header / -H (FirstLine mode)
// ============================================================================

#[test]
fn filter_header_firstline_basic() {
    let input = "col1\tcol2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "col1\tcol2");
    assert_eq!(lines[1], "1\ta");
}

#[test]
fn filter_header_short_flag() {
    let input = "col1\tcol2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "-H", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "col1\tcol2");
}

#[test]
fn filter_header_firstline_with_empty_lines() {
    // FirstLine mode now takes the first line even if empty
    let input = "\n\ncol1\tcol2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    // First line is empty, so it's treated as header (no column names for field resolution)
    // Uses numeric field index 2 for filtering
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], ""); // Empty header line
    assert_eq!(lines[1], "1\ta");
}

#[test]
fn filter_header_firstline_with_count() {
    let input = "col1\tcol2\n1\ta\n2\tb\n3\ta\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--count", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    // Should output count only, no header
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn filter_header_firstline_with_label() {
    let input = "col1\tcol2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--label", "match", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "col1\tcol2\tmatch");
    assert_eq!(lines[1], "1\ta\t1");
    assert_eq!(lines[2], "2\tb\t0");
}

// ============================================================================
// Tests for --header-lines (LinesN mode)
// ============================================================================

#[test]
fn filter_header_lines_n_basic() {
    let input = "header1\nheader2\ncol1\tcol2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header-lines", "2", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Only the last header line (col1\tcol2) is written as header
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "header2");
    assert_eq!(lines[1], "1\ta");
}

#[test]
fn filter_header_lines_n_with_count() {
    let input = "h1\nh2\ncol1\tcol2\n1\ta\n2\tb\n3\ta\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header-lines",
            "2",
            "--count",
            "--str-eq",
            "2:a",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "2");
}

// ============================================================================
// Tests for --header-hash (HashLines mode)
// ============================================================================

#[test]
fn filter_header_hash_basic() {
    // HashLines mode: captures hash lines as header, but no column names
    // Uses numeric field indices for filtering
    // Writes the last hash line as header output
    let input = "# comment 1\n# comment 2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header-hash", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    // HashLines mode: hash lines are header, last hash line is written
    // Data rows are filtered using numeric field indices
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "# comment 2");
    assert_eq!(lines[1], "1\ta");
}

#[test]
fn filter_header_hash_no_hash_lines() {
    // No hash lines - no header detected, behaves like no header mode
    let input = "1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header-hash", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    // No hash lines found, so behaves like no header
    // All lines treated as data, filter applied, no header output
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "1\ta");
}

// ============================================================================
// Tests for --header-hash1 (HashLines1 mode)
// ============================================================================

#[test]
fn filter_header_hash1_basic() {
    let input = "# comment 1\n# comment 2\ncol1\tcol2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header-hash1", "--str-eq", "2:a"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // HashLines1 mode: hash lines + next line (column names) are header
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "col1\tcol2");
    assert_eq!(lines[1], "1\ta");
}

#[test]
fn filter_header_hash1_with_label() {
    let input = "# comment\ncol1\tcol2\n1\ta\n2\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header-hash1",
            "--label",
            "result",
            "--str-eq",
            "2:a",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "col1\tcol2\tresult");
    assert_eq!(lines[1], "1\ta\t1");
    assert_eq!(lines[2], "2\tb\t0");
}

// ============================================================================
// Tests for conflicting header options
// ============================================================================

#[test]
fn filter_header_conflicting_options_error() {
    let input = "col1\tcol2\n1\ta\n";
    let (stdout, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--header-lines",
            "2",
            "--str-eq",
            "2:a",
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());
    // The error message should indicate conflicting options
    assert!(
        stderr.contains("only one of")
            || stderr.contains("conflict")
            || stderr.contains("cannot be used together"),
        "Expected error message about conflicting options, got: {}",
        stderr
    );
}

// ============================================================================
// Tests for multi-file input with headers
// ============================================================================

#[test]
fn filter_header_multifile_only_first_header() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "col1\tcol2").unwrap();
    writeln!(file1, "1\ta").unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "col1\tcol2").unwrap();
    writeln!(file2, "2\tb").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--str-eq", "2:a", path1, path2])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Only first file's header should be output
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "col1\tcol2");
    assert_eq!(lines[1], "1\ta");
}

// ============================================================================
// Tests for field name resolution with different header modes
// ============================================================================

#[test]
fn filter_header_hash1_field_by_name() {
    let input = "# comment\nname\tvalue\nAlice\t100\nBob\t200\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header-hash1", "--gt", "value:150"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "name\tvalue");
    assert_eq!(lines[1], "Bob\t200");
}

#[test]
fn filter_header_lines_n_field_by_name() {
    let input = "# meta\nname\tvalue\nAlice\t100\nBob\t200\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header-lines", "2", "--gt", "value:150"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "name\tvalue");
    assert_eq!(lines[1], "Bob\t200");
}
