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

#[test]
fn keep_header_empty_file_sort() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/emptyfile.txt",
            "--",
            tva_bin,
            "sort",
        ])
        .run();
    assert_eq!(stdout, "");
}

#[test]
fn keep_header_empty_file_and_input1_sort() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/emptyfile.txt",
            "tests/data/keep_header/input1.csv",
            "--",
            tva_bin,
            "sort",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.1,30,green|blue\n\
input1.txt.2,20,緑|青\n\
input1.txt.3,10,绿色|蓝色\n";

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_headeronly_file_sort() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input_headeronly.csv",
            "--",
            tva_bin,
            "sort",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);
    assert_eq!(stdout, "file.row,field1,field2,field3\n");
}

#[test]
fn keep_header_headeronly_file_and_input1_sort() {
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input_headeronly.csv",
            "tests/data/keep_header/input1.csv",
            "--",
            tva_bin,
            "sort",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    // headeronly.csv has header. input1.csv has same header.
    // keep-header should print header from headeronly.csv.
    // then pipe rest (empty) from headeronly.csv and body (3 lines) from input1.csv to sort.
    let expected = "file.row,field1,field2,field3\n\
input1.txt.1,30,green|blue\n\
input1.txt.2,20,緑|青\n\
input1.txt.3,10,绿色|蓝色\n";

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_oneblankline_cat() {
    // oneblankline.txt contains a single blank line.
    // If it's treated as header, it's printed.
    // Since it's just "\n", the behavior depends on whether we consider it empty or not.
    // tsv-utils considers it a header.
    let tva_bin = env!("CARGO_BIN_EXE_tva");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/oneblankline.txt",
            "--",
            tva_bin,
            "select", // Use select/cat to pass through
            "-f",
            "1-",
        ])
        .run();
    // In tva, read_line preserves \n.
    // The file has 1 line: "\n".
    // It's the first line, so it's the header.
    // It is printed directly.
    // The rest is empty.
    assert_eq!(stdout, "\n");
}

#[test]
fn keep_header_stdin_sort() {
    let input = std::fs::read_to_string("tests/data/keep_header/input1.csv").unwrap();
    let tva_bin = env!("CARGO_BIN_EXE_tva");

    let (stdout, _) = TvaCmd::new()
        .args(&["keep-header", "--", tva_bin, "sort"])
        .stdin(input)
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.1,30,green|blue\n\
input1.txt.2,20,緑|青\n\
input1.txt.3,10,绿色|蓝色\n";

    assert_eq!(stdout, expected);
}

#[test]
fn keep_header_stdin_pipe_complex() {
    // cat input1.csv | keep-header input2.csv - -- sort ...
    // input2 has header + 3 lines.
    // stdin (input1) has header + 3 lines.
    // Header comes from input2.
    // input2 body (3 lines) + input1 body (3 lines) -> sorted.

    let input1 = std::fs::read_to_string("tests/data/keep_header/input1.csv").unwrap();
    let tva_bin = env!("CARGO_BIN_EXE_tva");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "keep-header",
            "tests/data/keep_header/input2.csv",
            "-",
            "--",
            tva_bin,
            "sort",
            "-t",
            ",",
            "-k",
            "2",
            "-n",
        ])
        .stdin(input1)
        .run();
    let stdout = normalize_newlines(&stdout);

    // Sorted by 2nd column (numeric):
    // input1: 30, 20, 10
    // input2: 15, 25, 35
    // Sorted: 10(input1.3), 15(input2.1), 20(input1.2), 25(input2.2), 30(input1.1), 35(input2.3)
    let expected = "file.row,field1,field2,field3\n\
input1.txt.3,10,绿色|蓝色\n\
input2.txt.1,15,green|blue\n\
input1.txt.2,20,緑|青\n\
input2.txt.2,25,grün|blau\n\
input1.txt.1,30,green|blue\n\
input2.txt.3,35,vihreä|sininen\n";

    assert_eq!(stdout, expected);
}
