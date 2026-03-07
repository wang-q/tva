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
