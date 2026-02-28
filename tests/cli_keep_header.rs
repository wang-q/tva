#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

fn read_golden(name: &str) -> String {
    let path = format!("tests/data/keep_header/{}", name);
    std::fs::read_to_string(path).unwrap()
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn keep_header_single_file_sort() {
    let expected = read_golden("gold_single_sort.txt");
    let tva_bin = env!("CARGO_BIN_EXE_tva");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input1.csv",
            "--",
            tva_bin,
            "sort",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_multi_file_sort() {
    let expected = read_golden("gold_multi_sort.txt");
    let tva_bin = env!("CARGO_BIN_EXE_tva");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input1.csv",
            "tests/data/keep_header/input2.csv",
            "--",
            tva_bin,
            "sort",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_multi_line_header_with_lines_option() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&["keep-header", "--lines", "2", "-", "--", tva_bin, "sort"])
        .stdin(read_golden("multi_header.txt"))
        .run();
    let stdout = normalize_newlines(&stdout);

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2);
    assert_eq!(lines[0], "H1");
    assert_eq!(lines[1], "H2");
}

#[test]
fn keep_header_single_file_sort_reverse() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input1.csv",
            "--",
            tva_bin,
            "sort",
            "-r",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.2,20,緑|青\n\
input1.txt.1,30,green|blue\n";

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_single_file_sort_numeric_second_field() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input1.csv",
            "--",
            tva_bin,
            "sort",
            "-t",
            ",",
            "-k",
            "2",
            "-n",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.2,20,緑|青\n\
input1.txt.1,30,green|blue\n";

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_input1_twice_numeric_second_field() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input1.csv",
            "tests/data/keep_header/input1.csv",
            "--",
            tva_bin,
            "sort",
            "-t",
            ",",
            "-k",
            "2",
            "-n",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.2,20,緑|青\n\
input1.txt.2,20,緑|青\n\
input1.txt.1,30,green|blue\n\
input1.txt.1,30,green|blue\n";

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_missing_separator() {
    let (_, stderr) = TvaCmd::new().args(&["keep-header", "sort"]).run_fail();
    assert!(stderr.contains("required arguments were not provided"));
}

#[test]
fn keep_header_command_fail() {
    TvaCmd::new()
        .args(&["keep-header", "--", "non_existent_command_12345"])
        .run_fail();
}

#[test]
fn keep_header_lines_zero() {
    let input = "h\nd\n";
    let tva_bin = env!("CARGO_BIN_EXE_tva");

    let (stdout, _) = TvaCmd::new()
        .args(&["keep-header", "-n", "0", "--", tva_bin, "select", "-f", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("h\nd\n"));
}

#[test]
fn keep_header_file_open_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["keep-header", "non_existent_file_keep.tsv", "--", "sort"])
        .run_fail();

    assert!(stderr.contains("could not open"));
}
