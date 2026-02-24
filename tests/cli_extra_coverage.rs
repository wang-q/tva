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
