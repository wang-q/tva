use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_from_csv_invalid_delimiter_length() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from-csv")
        .arg("--delimiter")
        .arg("TAB")
        .write_stdin("a,b\n1,2\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("delimiter must be a single byte"));
}

#[test]
fn test_from_csv_empty_records() {
    // Tests L102-104: empty records (newlines) are skipped by the default CSV parser configuration.
    // The test confirms that empty lines do not appear in the output.
    let input = "a,b\n\n1,2\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from-csv")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("a\tb\n1\t2\n"));
}

#[test]
fn test_from_csv_stdin_error() {
    // Tests L120-126: invalid CSV from stdin
    // Case: inconsistent record length (Row 1: 2 fields, Row 2: 3 fields)
    let input = "a,b\n1,2,3\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from-csv")
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("tva from-csv: invalid CSV at line"));
}

#[test]
fn test_from_csv_file_error_no_line_info() {
    // This is hard to trigger with standard CSV parser as most errors have positions
    // But we can verify the file path is included in the error message for file inputs
    // Using a file that definitely has bad CSV structure
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from-csv")
        .arg("tests/data/from_csv/invalid1.csv")
        .assert()
        .failure()
        .stderr(predicate::str::contains("tva from-csv: invalid CSV in 'tests/data/from_csv/invalid1.csv'"));
}
