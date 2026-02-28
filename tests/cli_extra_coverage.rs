use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// -------------------------------------------------------------------------------------------------
// split.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_split_missing_args() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "either --lines-per-file/-l or --num-files/-n must be specified",
        ));
}

#[test]
fn test_split_lines_num_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .arg("-l")
        .arg("10")
        .arg("-n")
        .arg("2")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--lines-per-file/-l cannot be used with --num-files/-n",
        ));
}

#[test]
fn test_split_key_lines_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .arg("-k")
        .arg("1")
        .arg("-l")
        .arg("10")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--key-fields/-k is only supported with --num-files/-n",
        ));
}

#[test]
fn test_split_output_not_dir() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let file_path = temp.path().join("file");
    fs::write(&file_path, "content")?;

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .arg("-n")
        .arg("2")
        .arg("--dir")
        .arg(&file_path)
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("output path is not a directory"));

    Ok(())
}

#[test]
fn test_split_file_exists_no_append() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let dir = temp.path();

    // Create a file that split would try to create: split-1.tsv
    let file_path = dir.join("split-1.tsv");
    fs::write(&file_path, "existing")?;

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .arg("-n")
        .arg("1")
        .arg("--static-seed") // ensure deterministic behavior if rng is used
        .arg("--dir")
        .arg(dir)
        .write_stdin("row1\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("output file already exists"));

    Ok(())
}

#[test]
fn test_split_key_no_num() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .arg("-k")
        .arg("1")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "either --lines-per-file/-l or --num-files/-n must be specified",
        ));
}

// -------------------------------------------------------------------------------------------------
// from-csv.rs coverage tests
// -------------------------------------------------------------------------------------------------
