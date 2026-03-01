#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;

#[test]
fn reverse_basic() {
    let input = "1\n2\n3\n";
    let expected = "3\n2\n1\n";

    let (stdout, _) = TvaCmd::new().args(&["reverse"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_header() {
    let input = "H\n1\n2\n3\n";
    let expected = "H\n3\n2\n1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_stdin() {
    let input = "1\n2\n3\n";
    let expected = "3\n2\n1\n";

    let (stdout, _) = TvaCmd::new().args(&["reverse"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_empty() {
    let input = "";
    let expected = "";

    let (stdout, _) = TvaCmd::new().args(&["reverse"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_single_line() {
    let input = "1";
    let expected = "1";

    let (stdout, _) = TvaCmd::new().args(&["reverse"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_single_line_newline() {
    let input = "1\n";
    let expected = "1\n";

    let (stdout, _) = TvaCmd::new().args(&["reverse"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_two_lines_no_newline_at_end() {
    let input = "1\n2";
    // tva/tac behavior: last line without newline is printed first without newline,
    // then previous line with newline is printed.
    // "1\n" + "2" -> "2" + "1\n" = "21\n"
    let expected = "21\n";

    let (stdout, _) = TvaCmd::new().args(&["reverse"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_header_empty() {
    let input = "";
    let expected = "";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_header_single_line() {
    let input = "Header\n";
    let expected = "Header\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_header_two_lines() {
    let input = "Header\nData\n";
    let expected = "Header\nData\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_header_three_lines() {
    let input = "Header\n1\n2\n";
    let expected = "Header\n2\n1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_multi_file() {
    // Note: tva reverse currently reverses each file individually and concatenates the results.
    // This is different from `tac file1 file2` which reverses the concatenated stream (file2 then file1).
    // We are testing the current behavior of tva.

    let file1_content = "1\n2\n";
    let file2_content = "3\n4\n";

    // Using tempfiles
    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    fs::write(&file1_path, file1_content).unwrap();
    fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "reverse",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    // file1 reversed: "2\n1\n"
    // file2 reversed: "4\n3\n"
    // concatenated: "2\n1\n4\n3\n"
    let expected = "2\n1\n4\n3\n";

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_multi_file_header() {
    // With header, the first line of the FIRST file is treated as header.
    // Subsequent files are treated as data (no header check for them, or checking but header already printed).
    // Let's check implementation behavior:
    // `header_printed` is passed to `process_buffer`.
    // Once printed (in first file), it stays true.
    // So subsequent files are processed as pure data (reversed fully).

    let file1_content = "Header\n1\n";
    let file2_content = "2\n3\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    fs::write(&file1_path, file1_content).unwrap();
    fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "reverse",
            "--header",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    // file1: Header printed. "1\n" reversed -> "1\n". Output: "Header\n1\n"
    // file2: header_printed=true. "2\n3\n" reversed -> "3\n2\n".
    // Output total: "Header\n1\n3\n2\n"
    let expected = "Header\n1\n3\n2\n";

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_multi_file_header_empty_first() {
    // If first file is empty, header not found.
    // Second file starts with Header?
    // Implementation:
    // process_buffer(file1): empty -> returns. header_printed=false.
    // process_buffer(file2): finds header -> prints. header_printed=true.

    let file1_content = "";
    let file2_content = "Header\n1\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    fs::write(&file1_path, file1_content).unwrap();
    fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "reverse",
            "--header",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    // file1: nothing.
    // file2: Header found. Output: "Header\n1\n"
    let expected = "Header\n1\n";

    assert_eq!(stdout, expected);
}
