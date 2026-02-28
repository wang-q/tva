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
