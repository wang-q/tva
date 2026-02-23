use assert_cmd::cargo::cargo_bin_cmd;

fn extract_block(contents: &str, marker: &str) -> String {
    let mut lines = contents.lines();
    let mut in_block = false;
    let mut block_lines = Vec::new();

    while let Some(line) = lines.next() {
        if line.starts_with("====[") && line.ends_with("]====") {
            if in_block {
                break;
            }
            if line == marker {
                in_block = true;
                continue;
            }
        } else if in_block {
            if line.is_empty() {
                break;
            }
            block_lines.push(line);
        }
    }

    if block_lines.is_empty() {
        String::new()
    } else {
        let mut result = String::new();
        for (i, l) in block_lines.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(l);
        }
        result.push('\n');
        result
    }
}

#[test]
fn join_basic_inner_join_header_by_name() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-H")
        .arg("--filter-file")
        .arg("tests/data/join/filter_basic.tsv")
        .arg("--key-fields")
        .arg("id")
        .arg("--append-fields")
        .arg("fv1,fv2")
        .arg("tests/data/join/data_basic.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "id\tdv1\tfv1\tfv2\nk1\tx1\tv1a\tv1b\nk2\tx2\tv2a\tv2b\n"
    );

    Ok(())
}

#[test]
fn join_basic_from_golden_whole_line_key_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("--filter-file")
        .arg("tests/data/join/input1.tsv")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let golden = std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt")?;
    let expected = extract_block(
        &golden,
        "====[tsv-join --header --filter-file input1.tsv input2.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());

    Ok(())
}

#[test]
fn join_basic_inner_join_header_by_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-H")
        .arg("--filter-file")
        .arg("tests/data/join/filter_basic.tsv")
        .arg("-k")
        .arg("1")
        .arg("-a")
        .arg("2,3")
        .arg("tests/data/join/data_basic.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "id\tdv1\tfv1\tfv2\nk1\tx1\tv1a\tv1b\nk2\tx2\tv2a\tv2b\n"
    );

    Ok(())
}

#[test]
fn join_basic_from_golden_whole_line_key_header_short_opts() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("--key-fields")
        .arg("0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let golden = std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt")?;
    let expected = extract_block(
        &golden,
        "====[tsv-join --header -f input1.tsv --key-fields 0 input2.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());

    Ok(())
}

#[test]
fn join_error_delimiter_must_be_single_char() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--filter-file")
        .arg("tests/data/join/filter_basic.tsv")
        .arg("--delimiter")
        .arg("::")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: delimiter must be a single character"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_basic_exclude_header_whole_line_key() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("--exclude")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let golden = std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt")?;
    let expected = extract_block(
        &golden,
        "====[tsv-join --header -f input1.tsv --exclude input2.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());

    Ok(())
}

#[test]
fn join_basic_exclude_noheader_whole_line_key() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("--exclude")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let golden = std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt")?;
    let expected = extract_block(
        &golden,
        "====[tsv-join -f input1_noheader.tsv --exclude input2_noheader.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());

    Ok(())
}

#[test]
fn join_basic_allow_duplicate_keys_header_append_last_wins() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--filter-file")
        .arg("tests/data/join/input1.tsv")
        .arg("-a")
        .arg("5")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("tva join: line has 1 fields, but append index 5 is out of range")
        || stderr.contains("tva join: line has 1 fields, but key index 2 is out of range"),
        "stderr was: {}",
        stderr);

    Ok(())
}

#[test]
fn join_basic_allow_duplicate_keys_noheader_append_last_wins() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--filter-file")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-a")
        .arg("5")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();

    let golden = std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt")?;
    let expected = extract_block(
        &golden,
        "====[tsv-join -f input1_noheader.tsv -k 2 -a 5 --allow-duplicate-keys input2_noheader.tsv]====",
    );

    assert!(expected.starts_with("1\tggg\tUUU\t101b\t15b\t52"));
    assert!(
        stderr.contains("tva join: line has 1 fields, but append index 5 is out of range")
            || stderr.contains("tva join: line has 1 fields, but key index 2 is out of range"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_duplicate_keys_filter_header_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("4")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: duplicate key with different append values found in filter file"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}
