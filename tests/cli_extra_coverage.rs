use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// -------------------------------------------------------------------------------------------------
// select.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_select_fields_exclude_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .arg("--fields")
        .arg("1")
        .arg("--exclude")
        .arg("2")
        .write_stdin("a\tb\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--fields/-f and --exclude/-e cannot be used together",
        ));
}

#[test]
fn test_select_missing_args() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .write_stdin("a\tb\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "one of --fields/-f or --exclude/-e is required",
        ));
}

#[test]
fn test_select_invalid_delimiter() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .arg("-f")
        .arg("1")
        .arg("--delimiter")
        .arg("TAB")
        .write_stdin("a\tb\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "delimiter must be a single character",
        ));
}

#[test]
fn test_select_empty_selection() {
    let input = "a\tb\n1\t2\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .arg("--exclude")
        .arg("1,2")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\n\n")); // Two newlines for two rows
}

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
// Additional select.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_select_invalid_field_spec() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .arg("-f")
        .arg("0")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("field index must be >= 1"));
}

#[test]
fn test_select_exclude_with_header() {
    let input = "h1\th2\th3\nv1\tv2\tv3\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .arg("--header")
        .arg("--exclude")
        .arg("2")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("h1\th3\nv1\tv3"));
}

#[test]
fn test_select_exclude_by_name_with_header() {
    let input = "h1\th2\th3\nv1\tv2\tv3\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .arg("--header")
        .arg("--exclude")
        .arg("h2")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("h1\th3\nv1\tv3"));
}

// -------------------------------------------------------------------------------------------------
// from-csv.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_from_csv_invalid_delimiter_length() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from")
        .arg("csv")
        .arg("--delimiter")
        .arg("TAB")
        .write_stdin("a,b\n1,2\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("delimiter must be a single byte"));
}

#[test]
fn test_from_csv_empty_records() {
    let input = "a,b\n\n1,2\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from")
        .arg("csv")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("a\tb\n1\t2\n"));
}

#[test]
fn test_from_csv_stdin_error() {
    let input = "a,b\n1,2,3\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from")
        .arg("csv")
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "tva from csv: invalid CSV at line",
        ));
}

#[test]
fn test_from_csv_file_error_no_line_info() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/invalid1.csv")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "tva from csv: invalid CSV in 'tests/data/from_csv/invalid1.csv'",
        ));
}
