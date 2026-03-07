use crate::common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;

#[macro_use]
#[path = "common/mod.rs"]
mod common;

#[test]
fn fill_multi_file_header_handling() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t").unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n2\t").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["fill", "--header", "-f", "2", "-v", "0", path1, path2])
        .run();

    assert_eq!(stdout, "h1\th2\n1\t0\n2\t0\n");
}
