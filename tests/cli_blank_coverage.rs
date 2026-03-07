use crate::common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;

#[macro_use]
#[path = "common/mod.rs"]
mod common;

#[test]
fn blank_multi_file_header_handling() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t2").unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n3\t4").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["blank", "--header", "-f", "1", path1, path2])
        .run();

    // Should output header once, then data from both
    assert_eq!(stdout, "h1\th2\n1\t2\n3\t4\n");
}

#[test]
fn blank_empty_file_handling() {
    let file1 = NamedTempFile::new().unwrap();
    // Empty file
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n1\t2").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["blank", "--header", "-f", "1", path1, path2])
        .run();

    // First file empty -> skip.
    // Second file -> write header, write data.
    assert_eq!(stdout, "h1\th2\n1\t2\n");
}
