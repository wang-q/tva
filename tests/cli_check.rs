use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn check_valid_ctg() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("check")
        .arg("tests/genome/ctg.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("4 lines, 6 fields"));

    Ok(())
}

#[test]
fn check_empty_input() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("check").write_stdin("").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("0 lines, 0 fields"));

    Ok(())
}

#[test]
fn check_simple_matrix() -> anyhow::Result<()> {
    let input = "A\t1\t!\nB\t2\t@\nC\t3\t#\nD\t4\t$\nE\t5\t%\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("check").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("5 lines, 3 fields"));

    Ok(())
}

#[test]
fn check_invalid_structure_from_stdin() -> anyhow::Result<()> {
    let input = "a\tb\tc\n1\t2\n";

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("check").write_stdin(input);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("line 2 (2 fields):")
                .and(predicate::str::contains(
                    "tva check: structure check failed: line 2 has 2 fields (expected 3)",
                )),
        );

    Ok(())
}

#[test]
fn check_empty_line_zero_fields() -> anyhow::Result<()> {
    let input = "x\ty\n\nu\tv\n";

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("check").write_stdin(input);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("line 2 (0 fields):").and(predicate::str::contains(
                "tva check: structure check failed: line 2 has 0 fields (expected 2)",
            )),
        );

    Ok(())
}
