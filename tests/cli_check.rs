#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

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
