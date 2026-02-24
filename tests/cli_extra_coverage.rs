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
        .stderr(predicate::str::contains("--fields/-f and --exclude/-e cannot be used together"));
}

#[test]
fn test_select_missing_args() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("select")
        .write_stdin("a\tb\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("one of --fields/-f or --exclude/-e is required"));
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
        .stderr(predicate::str::contains("delimiter must be a single character"));
}

#[test]
fn test_select_empty_selection() {
    // If we select a field that doesn't exist, it might return empty string or error depending on implementation.
    // But here we want to trigger L254: if selected_indices.is_empty()
    // This happens if we exclude all fields.
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
// sort.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_sort_invalid_delimiter() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sort")
        .arg("--delimiter")
        .arg("TAB")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("delimiter must be a single byte"));
}

#[test]
fn test_sort_invalid_key() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sort")
        .arg("--key")
        .arg("0") // 1-based index required
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("field index must be >= 1"));
}

#[test]
fn test_sort_empty_input() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sort")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// -------------------------------------------------------------------------------------------------
// sample.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_sample_num_prob_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-n")
        .arg("10")
        .arg("-p")
        .arg("0.5")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--num/-n and --prob/-p cannot be used together"));
}

#[test]
fn test_sample_replace_prob_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-r")
        .arg("-p")
        .arg("0.5")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--replace/-r cannot be used with --prob/-p"));
}

#[test]
fn test_sample_replace_no_num() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-r")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--replace/-r requires --num/-n greater than 0"));
}

#[test]
fn test_sample_inorder_conflicts() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-i")
        .arg("-r") // Conflict
        .arg("-n")
        .arg("5")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--inorder/-i requires --num/-n without --replace/-r or --prob/-p"));
}

#[test]
fn test_sample_weight_prob_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-w")
        .arg("1")
        .arg("-p")
        .arg("0.5")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--weight-field/-w cannot be used with --prob/-p"));
}

#[test]
fn test_sample_invalid_prob() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-p")
        .arg("1.5")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid --prob/-p value"));
}

#[test]
fn test_sample_gen_random_inorder_conflicts() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("--gen-random-inorder")
        .arg("-n")
        .arg("10")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--gen-random-inorder cannot be combined with sampling options"));
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
        .stderr(predicate::str::contains("either --lines-per-file/-l or --num-files/-n must be specified"));
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
        .stderr(predicate::str::contains("--lines-per-file/-l cannot be used with --num-files/-n"));
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
        .stderr(predicate::str::contains("--key-fields/-k is only supported with --num-files/-n"));
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
        .stderr(predicate::str::contains("either --lines-per-file/-l or --num-files/-n must be specified"));
}

// -------------------------------------------------------------------------------------------------
// Additional sample.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_sample_weight_replace_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-w")
        .arg("1")
        .arg("-r")
        .arg("-n")
        .arg("10")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--weight-field/-w cannot be used with --replace/-r"));
}

#[test]
fn test_sample_key_no_prob() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-k")
        .arg("1")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--key-fields/-k requires --prob/-p"));
}

#[test]
fn test_sample_key_conflicts() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-k")
        .arg("1")
        .arg("-p")
        .arg("0.5")
        .arg("-n")
        .arg("10")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--key-fields/-k cannot be used with --num/-n"));
}

#[test]
fn test_sample_print_random_gen_random_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("--print-random")
        .arg("--gen-random-inorder")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--print-random cannot be used with --gen-random-inorder"));
}

#[test]
fn test_sample_print_random_replace_conflict() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("--print-random")
        .arg("-r")
        .arg("-n")
        .arg("10")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--print-random is not supported with --replace/-r"));
}

#[test]
fn test_sample_weight_index_out_of_range() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-w")
        .arg("5")
        .arg("-n")
        .arg("1")
        .write_stdin("a\tb\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("weight field index 5 out of range"));
}

#[test]
fn test_sample_weight_invalid_value() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-w")
        .arg("1")
        .arg("-n")
        .arg("1")
        .write_stdin("not_a_number\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("weight value `not_a_number` is not a valid number"));
}

#[test]
fn test_sample_key_index_out_of_range() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample")
        .arg("-k")
        .arg("5")
        .arg("-p")
        .arg("0.5")
        .write_stdin("a\tb\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("key field index 5 out of range"));
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
// Additional sort.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_sort_empty_key_part() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sort")
        .arg("-k")
        .arg("1,,2")
        .write_stdin("a\tb\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty key list element"));
}

// -------------------------------------------------------------------------------------------------
// check.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_check_multiple_files_fail_second() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "a\tb\n1\t2\n")?;
    fs::write(&file2, "a\tb\n1\t2\t3\n")?;

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("check")
        .arg(&file1)
        .arg(&file2)
        .assert()
        .failure()
        .stderr(predicate::str::contains("structure check failed"));
    Ok(())
}

#[test]
fn test_check_file_open_error() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("check")
        .arg("non_existent_file_check.tsv")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not open"));
}

// -------------------------------------------------------------------------------------------------
// keep-header.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_keep_header_missing_separator() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("keep-header")
        .arg("sort")
        .assert()
        .failure() // Now fails due to missing required command
        .stderr(predicate::str::contains("required arguments were not provided"));
}

#[test]
fn test_keep_header_command_fail() {
    // If the command doesn't exist, spawn should fail
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("keep-header")
        .arg("--")
        .arg("non_existent_command_12345")
        .assert()
        .failure();
}

#[test]
fn test_keep_header_lines_zero() {
    // Should default to 1
    let input = "h\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    // Use 'cat' (on unix) or 'type' (on windows)?
    // Wait, on windows 'cat' might not exist.
    // We can use 'tva' itself as a cat replacement: 'tva md' or similar?
    // Or just rely on standard tools available in environment.
    // The environment says "windows".
    // 'findstr' is common on windows. Or use `env!("CARGO_BIN_EXE_tva")` with `select -f 1-`.

    let tva_bin = env!("CARGO_BIN_EXE_tva");

    cmd.arg("keep-header")
        .arg("-n")
        .arg("0")
        .arg("--")
        .arg(tva_bin)
        .arg("select") // tva select -f 1 is basically cat for single column
        .arg("-f")
        .arg("1")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("h\nd\n"));
}

#[test]
fn test_keep_header_file_open_error() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("keep-header")
        .arg("non_existent_file_keep.tsv")
        .arg("--")
        .arg("sort")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not open"));
}

// -------------------------------------------------------------------------------------------------
// append.rs coverage tests
// -------------------------------------------------------------------------------------------------

#[test]
fn test_append_track_source() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("input3x2\tfield1\tfield2\tfield3"));
}

#[test]
fn test_append_source_header() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--source-header")
        .arg("filename")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("filename\tfield1\tfield2\tfield3"));
}

#[test]
fn test_append_file_mapping() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--file")
        .arg("custom_label=tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom_label\tfield1\tfield2\tfield3"));
}

#[test]
fn test_append_invalid_file_mapping() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--file")
        .arg("invalid_format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid --file value `invalid_format`; expected LABEL=FILE"));
}

#[test]
fn test_append_invalid_delimiter() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--delimiter")
        .arg("TooLong")
        .assert()
        .failure()
        .stderr(predicate::str::contains("delimiter must be a single byte"));
}

#[test]
fn test_append_stdin_default() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .write_stdin("field1\tfield2\nval1\tval2\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdin\tfield1\tfield2"));
}

#[test]
fn test_append_custom_delimiter() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .arg("--delimiter")
        .arg(":")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("input3x2:field1\tfield2\tfield3"));
}

#[test]
fn test_append_subdir_filename_label() {
    // Tests that path/to/file.tsv becomes label "file"
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("input3x2\tfield1"));
}

// -------------------------------------------------------------------------------------------------
// from-csv.rs coverage tests
// -------------------------------------------------------------------------------------------------

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
