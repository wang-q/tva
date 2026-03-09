#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn slice_keep_single_range() {
    let input = "h1\nr1\nr2\nr3\nr4\nr5\n";
    // Keep rows 2-4 (r2, r3, r4)
    // Original line numbers:
    // 1: h1
    // 2: r1
    // 3: r2
    // 4: r3
    // 5: r4
    // 6: r5

    let expected = "r2\nr3\nr4\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "-r", "3-5"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_keep_multiple_ranges() {
    let input = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n";
    // Keep 1-3 and 8-10
    let expected = "1\n2\n3\n8\n9\n10\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "-r", "1-3", "-r", "8-10"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_drop_single_row() {
    let input = "1\n2\n3\n4\n5\n";
    // Drop row 3
    let expected = "1\n2\n4\n5\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "-r", "3", "--invert"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_keep_header_drop_range() {
    let input = "Header\nData1\nData2\nData3\nData4\n";
    // Drop rows 1-3 (Header, Data1, Data2) but keep header with -H
    // So result should be: Header, Data3, Data4
    let expected = "Header\nData3\nData4\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "-r", "1-3", "--invert", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_keep_header_keep_range() {
    let input = "Header\nData1\nData2\nData3\nData4\n";
    // Keep rows 4-5 (Data3, Data4) plus Header
    let expected = "Header\nData3\nData4\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "-r", "4-5", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_open_ranges() {
    let input = "1\n2\n3\n4\n5\n";
    // 4- (4, 5)
    let expected = "4\n5\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "-r", "4-"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_start_ranges() {
    let input = "1\n2\n3\n4\n5\n";
    // -2 (1, 2)
    let expected = "1\n2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "-r", "-2"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_invalid_zero_index() {
    // Tests L74-75 and L85-86: Row index must be >= 1
    // Case 1: Single number 0
    let (_, stderr) = TvaCmd::new()
        .args(&["slice", "-r", "0"])
        .stdin("header\n")
        .run_fail();
    assert!(stderr.contains("Row index must be >= 1"));

    // Case 2: Range starting with 0 (0-5)
    let (_, stderr2) = TvaCmd::new()
        .args(&["slice", "-r", "0-5"])
        .stdin("header\n")
        .run_fail();
    assert!(stderr2.contains("Row index must be >= 1"));
}

#[test]
fn slice_invalid_range_order() {
    // Tests L77-78: Invalid range: end < start
    let (_, stderr) = TvaCmd::new()
        .args(&["slice", "-r", "5-2"])
        .stdin("header\n")
        .run_fail();
    assert!(stderr.contains("Invalid range: end < start"));
}

#[test]
fn slice_empty_ranges_behavior() {
    // Tests L140-147: Empty ranges list behavior
    let input = "1\n2\n3\n";

    // Case 1: Keep mode with no ranges -> Keep nothing
    // But wait, if keep_header is set, header should still be printed?
    // Let's test basic case first.
    let (stdout, _) = TvaCmd::new()
        .args(&["slice"]) // No -r provided
        .stdin(input)
        .run();
    assert_eq!(stdout, ""); // Keep nothing

    // Case 2: Drop mode (invert) with no ranges -> Drop nothing (Keep all)
    let (stdout2, _) = TvaCmd::new()
        .args(&["slice", "--invert"])
        .stdin(input)
        .run();
    assert_eq!(stdout2, "1\n2\n3\n");

    // Case 3: Keep mode with no ranges BUT with --header
    let (stdout3, _) = TvaCmd::new()
        .args(&["slice", "--header"])
        .stdin(input)
        .run();
    assert_eq!(stdout3, "1\n"); // Header kept, rest dropped
}

// Tests for different header modes

#[test]
fn slice_header_lines_n_mode() {
    // --header-lines N: Treat exactly N non-empty lines as header
    let input = "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\nData3\tData4\n";

    // Keep rows 4-5 (Data1, Data2 and Data3, Data4), header is first 3 lines
    // Line numbers: 1:#Comment1, 2:#Comment2, 3:Col1\tCol2, 4:Data1, 5:Data2, ...
    let expected = "# Comment 1\n# Comment 2\nCol1\tCol2\nData3\tData4\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "--header-lines", "3", "-r", "5"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_header_hash_mode() {
    // --header-hash: Treat consecutive '#' lines as header
    let input = "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\nData3\tData4\n";

    // Header is lines 1-2 (# comments), line 3 is column names, data starts at line 4
    // Keep row 4 (Data1 line) with header
    let expected = "# Comment 1\n# Comment 2\nData1\tData2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "--header-hash", "-r", "4"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_header_hash1_mode() {
    // --header-hash1: Treat '#' lines plus next line as header (for column names)
    let input = "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\nData3\tData4\n";

    // Header is lines 1-3 (# comments + column names line), data starts at line 4
    // Keep row 4 (Data1 line) with header
    let expected = "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "--header-hash1", "-r", "4"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_header_lines_n_with_invert() {
    // Test --header-lines with --invert (drop mode)
    let input = "Header1\nHeader2\nData1\nData2\nData3\n";

    // Header is lines 1-2, drop rows 1-3 (Header1, Header2, Data1), keep Data2, Data3
    // But since we have header mode, header should be preserved
    let expected = "Header1\nHeader2\nData3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "--header-lines", "2", "-r", "3-4", "--invert"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_header_hash_with_multiple_ranges() {
    // Test --header-hash with multiple ranges
    // Note: --header-hash only treats '#' lines as header, column names line is data
    let input = "# Comment\nCol1\tCol2\nRow1\nRow2\nRow3\nRow4\n";

    // Header is line 1 (# Comment), column names line 2 is DATA, data rows are 3-6
    // Keep rows 3 and 5 (Row1 and Row3) with header
    let expected = "# Comment\nRow1\nRow3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "--header-hash", "-r", "3", "-r", "5"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

// Multi-file input tests

#[test]
fn slice_multi_file_input() {
    // Test reading from multiple files
    let file1_content = "1\n2\n3\n";
    let file2_content = "4\n5\n6\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    std::fs::write(&file1_path, file1_content).unwrap();
    std::fs::write(&file2_path, file2_content).unwrap();

    // Keep rows 1-2 from each file
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-r",
            "1-2",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    // Each file processed independently with its own line numbering
    // File 1: rows 1, 2 -> "1\n2\n"
    // File 2: rows 1, 2 -> "4\n5\n"
    assert_eq!(stdout, "1\n2\n4\n5\n");
}

#[test]
fn slice_multi_file_with_header() {
    // Test multi-file input with header - header should only be written once
    let file1_content = "Header\nData1\nData2\n";
    let file2_content = "Header\nData3\nData4\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    std::fs::write(&file1_path, file1_content).unwrap();
    std::fs::write(&file2_path, file2_content).unwrap();

    // Keep row 2 (first data row) from each file, with header
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-H",
            "-r",
            "2",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    // Header should appear only once, then Data1 from file1 and Data3 from file2
    assert_eq!(stdout, "Header\nData1\nData3\n");
    assert_eq!(
        stdout.matches("Header").count(),
        1,
        "Header should appear exactly once"
    );
}

#[test]
fn slice_outfile_option() {
    // Test --outfile option
    let input = "1\n2\n3\n4\n5\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-r",
            "2-3",
            "--outfile",
            output_path.to_str().unwrap(),
        ])
        .stdin(input)
        .run();

    // stdout should be empty when using --outfile
    assert_eq!(stdout, "");

    // Check output file content
    let output_content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(output_content, "2\n3\n");
}
