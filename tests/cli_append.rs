use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

fn extract_block<F>(matcher: F) -> String
where
    F: Fn(&str) -> bool,
{
    let gold = fs::read_to_string("tests/data/append/gold_basic_tests_1.txt").unwrap();
    let lines: Vec<&str> = gold.lines().collect();

    let mut start_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if matcher(line) {
            start_idx = Some(i + 1);
            break;
        }
    }

    let start = start_idx.expect("Header not found in gold file");

    let mut end = lines.len();
    for i in start..lines.len() {
        let line = lines[i];
        if line.starts_with("====[tsv-append ")
            || line.starts_with("====[cat ")
            || line.starts_with("Help and Version printing")
        {
            end = i;
            break;
        }
    }

    let mut block: Vec<&str> = lines[start..end].to_vec();
    while matches!(block.last(), Some(l) if l.is_empty()) {
        block.pop();
    }

    if block.is_empty() {
        return String::new();
    }

    block.join("\n") + "\n"
}

fn expected_block(command: &str) -> String {
    let header = format!("====[tsv-append {}]====", command);
    extract_block(|line| line == header)
}

#[test]
fn append_basic_input3x2_input3x5_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input3x2.tsv input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_basic_input1x3_input1x4_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1x3.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_basic_four_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input3x2.tsv input1x3.tsv input3x5.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_basic_single_file_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_two_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header input3x2.tsv input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("--header")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_two_single_column_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-H input1x3.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-H")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_four_files_from_gold() -> anyhow::Result<()> {
    let expected =
        expected_block("-H input3x2.tsv input1x3.tsv input3x5.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-H")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_single_file_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-H input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-H")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_track_source_two_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--track-source input3x2.tsv input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("--track-source")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_track_source_two_single_column_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-t input1x3.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-t")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_track_source_four_files_from_gold() -> anyhow::Result<()> {
    let expected =
        expected_block("-t input3x2.tsv input1x3.tsv input3x5.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-t")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_track_source_single_file_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-t input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-t")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_and_track_source_two_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header --track-source input3x2.tsv input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("--header")
        .arg("--track-source")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_and_track_source_two_single_column_files_from_gold(
) -> anyhow::Result<()> {
    let expected = expected_block("-H -t input1x3.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-H")
        .arg("-t")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_and_track_source_four_files_from_gold() -> anyhow::Result<()> {
    let expected =
        expected_block("-H -t input3x2.tsv input1x3.tsv input3x5.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-H")
        .arg("-t")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_header_and_track_source_single_file_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-H -t input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-H")
        .arg("-t")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_source_header_two_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--source-header source input3x2.tsv input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("--source-header")
        .arg("source")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_source_header_two_single_column_files_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--s source input1x3.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("--source-header")
        .arg("source")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_source_header_four_files_from_gold() -> anyhow::Result<()> {
    let expected =
        expected_block("-s source input3x2.tsv input1x3.tsv input3x5.tsv input1x4.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-s")
        .arg("source")
        .arg("tests/data/append/input3x2.tsv")
        .arg("tests/data/append/input1x3.tsv")
        .arg("tests/data/append/input3x5.tsv")
        .arg("tests/data/append/input1x4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn append_source_header_single_file_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-s source input3x5.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("append")
        .arg("-s")
        .arg("source")
        .arg("tests/data/append/input3x5.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}
