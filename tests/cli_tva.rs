use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command;
use tempfile::TempDir; // Run programs

#[test]
fn command_invalid() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("tva")?;
    cmd.arg("foobar");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("recognized"));

    Ok(())
}

#[test]
fn command_md() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("tva")?;
    let output = cmd
        .arg("md")
        .arg("tests/genome/ctg.range.tsv")
        .arg("--num")
        .arg("-c")
        .arg("2")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(
        stdout.contains("| -----: | :--------: | --------------- |"),
        "separator"
    );
    assert!(stdout.contains("| 130218 |  ctg:I:2   | I:100001-230218 |"));

    let mut cmd = Command::cargo_bin("tva")?;
    let output = cmd
        .arg("md")
        .arg("tests/genome/ctg.range.tsv")
        .arg("--fmt")
        .arg("--digits")
        .arg("2")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(
        stdout.contains("| ---------: | ---------- | --------------- |"),
        "separator"
    );
    assert!(stdout.contains("| 130,218.00 | ctg:I:2    | I:100001-230218 |"));

    Ok(())
}
