#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn check_valid_ctg() -> anyhow::Result<()> {
    let (stdout, _) = TvaCmd::new().args(&["check", "tests/genome/ctg.tsv"]).run();

    assert!(stdout.contains("4 lines, 6 fields"));

    Ok(())
}

#[test]
fn check_empty_input() -> anyhow::Result<()> {
    let (stdout, _) = TvaCmd::new().stdin("").args(&["check"]).run();

    assert!(stdout.contains("0 lines, 0 fields"));

    Ok(())
}

#[test]
fn check_simple_matrix() -> anyhow::Result<()> {
    let input = "A\t1\t!\nB\t2\t@\nC\t3\t#\nD\t4\t$\nE\t5\t%\n";

    let (stdout, _) = TvaCmd::new().stdin(input).args(&["check"]).run();

    assert!(stdout.contains("5 lines, 3 fields"));

    Ok(())
}

#[test]
fn check_invalid_structure_from_stdin() -> anyhow::Result<()> {
    let input = "a\tb\tc\n1\t2\n";

    let (_, stderr) = TvaCmd::new().stdin(input).args(&["check"]).run_fail();

    assert!(stderr.contains("line 2 (2 fields):"));
    assert!(stderr.contains(
        "tva check: structure check failed: line 2 has 2 fields (expected 3)"
    ));

    Ok(())
}

#[test]
fn check_empty_line_zero_fields() -> anyhow::Result<()> {
    let input = "x\ty\n\nu\tv\n";

    let (_, stderr) = TvaCmd::new().stdin(input).args(&["check"]).run_fail();

    assert!(stderr.contains("line 2 (0 fields):"));
    assert!(stderr.contains(
        "tva check: structure check failed: line 2 has 0 fields (expected 2)"
    ));

    Ok(())
}

#[test]
fn check_multiple_files_fail_second() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "a\tb\n1\t2\n")?;
    fs::write(&file2, "a\tb\n1\t2\t3\n")?;

    let file1_str = file1.to_str().unwrap();
    let file2_str = file2.to_str().unwrap();

    let (_, stderr) = TvaCmd::new()
        .args(&["check", file1_str, file2_str])
        .run_fail();

    assert!(stderr.contains("structure check failed"));

    Ok(())
}

#[test]
fn check_file_open_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["check", "non_existent_file_check.tsv"])
        .run_fail();

    assert!(stderr.contains("could not open"));
}
