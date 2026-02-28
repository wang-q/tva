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
