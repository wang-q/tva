use crate::common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;

#[macro_use]
#[path = "common/mod.rs"]
mod common;

#[test]
fn append_basic_stdin() {
    let (stdout, _) = TvaCmd::new().stdin("a\tb\n1\t2\n").args(&["append"]).run();
    assert_eq!(stdout, "a\tb\n1\t2\n");
}

#[test]
fn append_track_source_stdin() {
    let (stdout, _) = TvaCmd::new()
        .stdin("a\tb\n1\t2\n")
        .args(&["append", "--track-source"])
        .run();
    assert_eq!(stdout, "stdin\ta\tb\nstdin\t1\t2\n");
}

#[test]
fn append_source_header() {
    let (stdout, _) = TvaCmd::new()
        .stdin("a\tb\n1\t2\n")
        .args(&["append", "--source-header", "SRC"])
        .run();
    assert_eq!(stdout, "SRC\ta\tb\nstdin\t1\t2\n");
}

#[test]
fn append_file_label() {
    // LABEL=FILE where FILE is "-" (stdin)
    let (stdout, _) = TvaCmd::new()
        .stdin("a\tb\n1\t2\n")
        .args(&["append", "--file", "mysource=-"])
        .run();
    assert_eq!(stdout, "mysource\ta\tb\nmysource\t1\t2\n");
}

#[test]
fn append_error_file_format() {
    let (_, stderr) = TvaCmd::new()
        .args(&["append", "--file", "nolabel"])
        .run_fail();
    assert!(stderr.contains("invalid --file value"));
}

#[test]
fn append_error_delimiter_length() {
    let (_, stderr) = TvaCmd::new()
        .args(&["append", "--delimiter", "tab"])
        .run_fail();
    assert!(stderr.contains("delimiter must be a single byte"));
}

#[test]
fn append_header_handling() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t2").unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n3\t4").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--header", path1, path2])
        .run();

    assert_eq!(stdout, "h1\th2\n1\t2\n3\t4\n");
}

#[test]
fn append_header_handling_with_source() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t2").unwrap();
    let path1 = file1.path().to_str().unwrap();
    let name1 = std::path::Path::new(path1)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n3\t4").unwrap();
    let path2 = file2.path().to_str().unwrap();
    let name2 = std::path::Path::new(path2)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--source-header", "SRC", path1, path2])
        .run();

    let expected = format!("SRC\th1\th2\n{}\t1\t2\n{}\t3\t4\n", name1, name2);
    assert_eq!(stdout, expected);
}
