#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn sort_invalid_delimiter() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "--delimiter", "TAB"])
        .stdin("a\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains("delimiter must be a single byte"));
}

#[test]
fn sort_invalid_key() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "--key", "0"])
        .stdin("a\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn sort_empty_input() {
    let (stdout, _) = TvaCmd::new().args(&["sort"]).stdin("").run();

    assert!(stdout.is_empty());
}

#[test]
fn sort_default_lexicographic_single_key() {
    let input = "a\t2\nc\t1\nb\t3\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    assert_eq!(stdout, "a\t2\nb\t3\nc\t1\n");
}

#[test]
fn sort_numeric_reverse_single_key() {
    let input = "a\t2\nc\t10\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n", "-r"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "c\t10\nb\t3\na\t2\n");
}

#[test]
fn sort_multiple_keys() {
    let input = "a\t2\nc\t1\nb\t1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2,1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "b\t1\nc\t1\na\t2\n");
}

#[test]
fn sort_default_all_columns_when_no_key() {
    let input = "b\t2\nb\t1\na\t3\n";

    let (stdout, _) = TvaCmd::new().args(&["sort"]).stdin(input).run();

    assert_eq!(stdout, "a\t3\nb\t1\nb\t2\n");
}

#[test]
fn sort_respects_custom_delimiter() {
    let input = "a,2\nc,1\nb,3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-t", ",", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "a,2\nb,3\nc,1\n");
}

#[test]
fn sort_numeric_with_non_numeric_values() {
    let input = "x\n10\nLETTER\n2\n1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "1", "-n"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "x\nLETTER\n1\n2\n10\n");
}

#[test]
fn sort_reverse_lexicographic_single_key() {
    let input = "a\t2\nc\t1\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "1", "-r"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "c\t1\nb\t3\na\t2\n");
}

#[test]
fn sort_lexicographic_file_names() {
    let input = "file2.txt\na\nfile10.txt\nfile1.txt\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    assert_eq!(stdout, "a\nfile1.txt\nfile10.txt\nfile2.txt\n");
}

#[test]
fn sort_with_header() {
    let input = "name\tval\nc\t1\na\t2\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "--header", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "name\tval\na\t2\nb\t3\nc\t1\n");
}

#[test]
fn sort_empty_key_part() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "-k", "1,,2"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains("empty key list element"));
}

// Tests for new header modes

#[test]
fn sort_with_header_lines_n() {
    // LinesN mode: first N lines are treated as header (no column names line)
    let input = "# Comment 1\n# Comment 2\nc\t1\na\t2\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "--header-lines", "2", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "# Comment 1\n# Comment 2\na\t2\nb\t3\nc\t1\n");
}

#[test]
fn sort_with_header_hash() {
    let input = "# Comment 1\n# Comment 2\nc\t1\na\t2\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "--header-hash", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "# Comment 1\n# Comment 2\na\t2\nb\t3\nc\t1\n");
}

#[test]
fn sort_with_header_hash1() {
    let input = "# Comment 1\n# Comment 2\nname\tval\nc\t1\na\t2\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "--header-hash1", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(
        stdout,
        "# Comment 1\n# Comment 2\nname\tval\na\t2\nb\t3\nc\t1\n"
    );
}

#[test]
fn sort_with_header_hash1_no_hash_lines() {
    // When no hash lines exist, should use first line as column names
    let input = "name\tval\nc\t1\na\t2\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "--header-hash1", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "name\tval\na\t2\nb\t3\nc\t1\n");
}

#[test]
fn sort_header_modes_mutually_exclusive() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "--header", "--header-lines", "2", "-k", "1"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains(
        "only one of --header, --header-lines, --header-hash, --header-hash1"
    ));
}

#[test]
fn sort_with_empty_lines() {
    // Test handling of empty lines in input
    let input = "b\t2\n\na\t1\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    // Empty line should be preserved and sorted first (empty string < "a")
    assert_eq!(stdout, "\na\t1\nb\t2\n");
}

#[test]
fn sort_header_only_no_data() {
    // Test when input has only header and no data rows
    let input = "name\tvalue\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "--header", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "name\tvalue\n");
}

#[test]
fn sort_header_lines_n_no_data() {
    // Test --header-lines with no data rows
    let input = "# Comment 1\n# Comment 2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "--header-lines", "2", "-k", "1"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "# Comment 1\n# Comment 2\n");
}

#[test]
fn sort_outfile() {
    // Test output to file using --outfile
    let input = "c\t3\na\t1\nb\t2\n";
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("sorted.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "1", "-o", outfile.to_str().unwrap()])
        .stdin(input)
        .run();

    // stdout should be empty when writing to file
    assert!(stdout.is_empty());

    // Verify file contents
    let contents = std::fs::read_to_string(&outfile).unwrap();
    assert_eq!(contents, "a\t1\nb\t2\nc\t3\n");
}

#[test]
fn sort_outfile_with_header() {
    // Test output to file with header
    let input = "name\tvalue\nc\t3\na\t1\n";
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("sorted.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sort",
            "--header",
            "-k",
            "1",
            "-o",
            outfile.to_str().unwrap(),
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let contents = std::fs::read_to_string(&outfile).unwrap();
    assert_eq!(contents, "name\tvalue\na\t1\nc\t3\n");
}

#[test]
fn sort_single_row() {
    // Test sorting with only one data row
    let input = "b\t2\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    assert_eq!(stdout, "b\t2\n");
}

#[test]
fn sort_numeric_with_floats() {
    // Test numeric sorting with floating point numbers
    let input = "a\t3.14\nb\t2.71\nc\t1.41\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "c\t1.41\nb\t2.71\na\t3.14\n");
}

#[test]
fn sort_numeric_with_negative_numbers() {
    // Test numeric sorting with negative numbers
    let input = "a\t-5\nb\t-10\nc\t0\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "b\t-10\na\t-5\nc\t0\n");
}

#[test]
fn sort_reverse_numeric() {
    // Test reverse numeric sorting
    let input = "a\t1\nb\t3\nc\t2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n", "-r"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "b\t3\nc\t2\na\t1\n");
}

#[test]
fn sort_key_range() {
    // Test sorting with key range like "1-2"
    let input = "b\t2\na\t1\na\t2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "1-2"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "a\t1\na\t2\nb\t2\n");
}

#[test]
fn sort_key_with_duplicate_indices() {
    // Test that duplicate indices in key are handled (only first occurrence used)
    let input = "c\t3\na\t1\nb\t2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "1,1"]) // duplicate key should be deduplicated
        .stdin(input)
        .run();

    assert_eq!(stdout, "a\t1\nb\t2\nc\t3\n");
}

#[test]
fn sort_unicode_values() {
    // Test sorting with unicode characters
    let input = "中\t2\nあ\t1\n🎉\t3\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    // Unicode sort order based on byte values
    assert!(stdout.contains("あ"));
    assert!(stdout.contains("中"));
    assert!(stdout.contains("🎉"));
}

#[test]
fn sort_missing_fields_in_some_rows() {
    // Test sorting when some rows have fewer fields
    let input = "a\t2\nb\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    assert_eq!(stdout, "a\t2\nb\n");
}

#[test]
fn sort_with_whitespace_values() {
    // Test sorting values with leading/trailing whitespace
    let input = " b\t2\n a\t1\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    // Space (ASCII 32) comes before 'a' (ASCII 97)
    assert_eq!(stdout, " a\t1\n b\t2\n");
}

#[test]
fn sort_multiple_files_with_different_headers() {
    // Test multi-file input where files have different headers
    // Only first file's header should be used
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("file1.tsv");
    let file2 = dir.path().join("file2.tsv");

    std::fs::write(&file1, "# File1 header\nc\t1\na\t2\n").unwrap();
    std::fs::write(&file2, "# File2 header\nb\t3\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sort",
            "--header-lines",
            "1",
            "-k",
            "1",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    // Only first file's header should appear
    assert_eq!(stdout, "# File1 header\na\t2\nb\t3\nc\t1\n");
}

#[test]
fn sort_identical_values_stable() {
    // Test that sort is stable (preserves order of equal elements)
    let input = "b\t2\na\t1\na\t1\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "2"]).stdin(input).run();

    // Both 'a' rows should appear before 'b', and their relative order should be preserved
    assert_eq!(stdout, "a\t1\na\t1\nb\t2\n");
}

#[test]
fn sort_large_numeric_values() {
    // Test with large numbers
    let input = "a\t999999999\nb\t1000000000\nc\t1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "c\t1\na\t999999999\nb\t1000000000\n");
}

#[test]
fn sort_scientific_notation() {
    // Test numeric sorting with scientific notation
    let input = "a\t1e5\nb\t1e2\nc\t1e3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "b\t1e2\nc\t1e3\na\t1e5\n");
}

#[test]
fn sort_with_header_lines_n_multi_file() {
    // Create temp files for multi-file test
    // LinesN mode: first N lines are treated as header (no column names line)
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("file1.tsv");
    let file2 = dir.path().join("file2.tsv");

    std::fs::write(&file1, "# Comment 1\nc\t1\na\t2\n").unwrap();
    std::fs::write(&file2, "# Comment 1\nb\t3\nd\t4\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sort",
            "--header-lines",
            "1",
            "-k",
            "1",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    // Only first file's header should be written
    assert_eq!(stdout, "# Comment 1\na\t2\nb\t3\nc\t1\nd\t4\n");
}
