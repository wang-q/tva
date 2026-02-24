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
fn join_basic_line_buffered_header_filter_file() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--line-buffered")
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
fn join_basic_line_buffered_noheader_filter_file() -> anyhow::Result<()> {
    let mut cmd1 = cargo_bin_cmd!("tva");
    let output1 = cmd1
        .arg("join")
        .arg("--filter-file")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let output2 = cmd2
        .arg("join")
        .arg("--line-buffered")
        .arg("--filter-file")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    assert_eq!(stdout1, stdout2);

    Ok(())
}

#[test]
fn join_basic_line_buffered_header_key_fields_1() -> anyhow::Result<()> {
    let mut cmd1 = cargo_bin_cmd!("tva");
    let output1 = cmd1
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("--key-fields")
        .arg("1")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let output2 = cmd2
        .arg("join")
        .arg("--line-buffered")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("--key-fields")
        .arg("1")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    assert_eq!(stdout1, stdout2);

    Ok(())
}

#[test]
fn join_basic_line_buffered_noheader_key_fields_1() -> anyhow::Result<()> {
    let mut cmd1 = cargo_bin_cmd!("tva");
    let output1 = cmd1
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("--key-fields")
        .arg("1")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let output2 = cmd2
        .arg("join")
        .arg("--line-buffered")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("--key-fields")
        .arg("1")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    assert_eq!(stdout1, stdout2);

    Ok(())
}

#[test]
fn join_basic_line_buffered_header_data_fields() -> anyhow::Result<()> {
    let mut cmd1 = cargo_bin_cmd!("tva");
    let output1 = cmd1
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("--data-fields")
        .arg("2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let output2 = cmd2
        .arg("join")
        .arg("--line-buffered")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("--data-fields")
        .arg("2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    assert_eq!(stdout1, stdout2);

    Ok(())
}

#[test]
fn join_basic_line_buffered_header_allow_duplicate_keys_append() -> anyhow::Result<()> {
    let mut cmd1 = cargo_bin_cmd!("tva");
    let output1 = cmd1
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("5")
        .arg("--allow-duplicate-keys")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let output2 = cmd2
        .arg("join")
        .arg("--line-buffered")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("5")
        .arg("--allow-duplicate-keys")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    assert_eq!(stdout1, stdout2);

    Ok(())
}

#[test]
fn join_basic_line_buffered_header_allow_duplicate_keys_append_whole_line() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--line-buffered")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("3")
        .arg("-d")
        .arg("2")
        .arg("-a")
        .arg("0")
        .arg("--allow-duplicate-keys")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: field index must be >= 1 in `0`"),
        "stderr was: {}",
        stderr
    );

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
fn join_basic_write_all_left_outer_join_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("--allow-duplicate-keys")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("5")
        .arg("--write-all")
        .arg("NA")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "f1\tf2\tf3\tf4\tf5\tf5");

    let mut has_unmatched = false;
    for line in &lines[1..] {
        let fields: Vec<&str> = line.split('\t').collect();
        assert!(fields.len() == 5 || fields.len() == 6);
        if fields.len() == 6 && fields[5] == "NA" {
            has_unmatched = true;
        }
    }
    assert!(has_unmatched);

    Ok(())
}

#[test]
fn join_basic_write_all_multi_append_fields_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("--allow-duplicate-keys")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("4,5")
        .arg("--write-all")
        .arg("NA")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "f1\tf2\tf3\tf4\tf5\tf4\tf5");

    assert!(stdout.contains("27\txa\tgg\t44\t45\tNA\tNA"));

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
fn join_basic_write_all_left_outer_join_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("--allow-duplicate-keys")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("5")
        .arg("--write-all")
        .arg("NA")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("27\txa\tgg\t44\t45\tNA"));

    Ok(())
}

#[test]
fn join_basic_single_column_filter_header_key_only() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input_1x5.tsv")
        .arg("-k")
        .arg("1")
        .arg("tests/data/join/input1.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines[0], "f1\tf2\tf3\tf4\tf5");
    assert_eq!(lines.len(), 2);
    assert!(lines[1].starts_with("3\tnnn\tGGG\t336"));

    Ok(())
}

#[test]
fn join_basic_single_column_filter_header_data_and_append() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input_1x5.tsv")
        .arg("-k")
        .arg("1")
        .arg("-d")
        .arg("2")
        .arg("-a")
        .arg("1")
        .arg("tests/data/join/input1.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let expected = concat!(
        "f1\tf2\tf3\tf4\tf5\tfa\n",
        "1\tggg\tUUU\t101\t15\tggg\n",
        "5\tggg\tCCC\t5734\t52\tggg\n",
        "9\tv\tvv\t97\t91\tv\n",
    );

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn join_basic_alternate_delimiter_header_key_only() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--delimiter")
        .arg(":")
        .arg("--header")
        .arg("-k")
        .arg("1")
        .arg("-f")
        .arg("tests/data/join/input_2x3_colon.tsv")
        .arg("tests/data/join/input_5x4_colon.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let expected = concat!(
        "Field A:Field B:Field C:Field D:Field E\n",
        "13:hello world:fast:432:303\n",
        "101:501:432:12:13\n",
    );

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn join_basic_alternate_delimiter_header_with_data_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--delimiter")
        .arg(":")
        .arg("--header")
        .arg("-k")
        .arg("1")
        .arg("-d")
        .arg("5")
        .arg("-f")
        .arg("tests/data/join/input_2x3_colon.tsv")
        .arg("tests/data/join/input_5x4_colon.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let expected = concat!(
        "Field A:Field B:Field C:Field D:Field E\n",
        "55:\tabc:501:892:101\n",
        "101:501:432:12:13\n",
    );

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn join_basic_alternate_delimiter_header_multi_key_append() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--delimiter")
        .arg(":")
        .arg("--header")
        .arg("-k")
        .arg("1,2")
        .arg("-d")
        .arg("5,3")
        .arg("-a")
        .arg("1")
        .arg("-f")
        .arg("tests/data/join/input_2x3_colon.tsv")
        .arg("tests/data/join/input_5x4_colon.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout.trim_end(),
        "Field A:Field B:Field C:Field D:Field E:col a"
    );

    Ok(())
}

#[test]
fn join_basic_alternate_delimiter_header_named_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--delimiter")
        .arg(":")
        .arg("--header")
        .arg("-d")
        .arg("col a,col b")
        .arg("-k")
        .arg("Field E,Field C")
        .arg("-a")
        .arg("Field B")
        .arg("-f")
        .arg("tests/data/join/input_5x4_colon.tsv")
        .arg("tests/data/join/input_2x3_colon.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.trim_end(), "col a:col b:Field B");

    Ok(())
}

#[test]
fn join_basic_empty_filter_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input_emptyfile.tsv")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.is_empty());

    Ok(())
}

#[test]
fn join_basic_empty_filter_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-H")
        .arg("-f")
        .arg("tests/data/join/input_emptyfile.tsv")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "f1\tf2\tf3\tf4\tf5\n");

    Ok(())
}

#[test]
fn join_basic_empty_data_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("tests/data/join/input_emptyfile.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.is_empty());

    Ok(())
}

#[test]
fn join_basic_empty_data_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-H")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("tests/data/join/input_emptyfile.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.is_empty());

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
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("5")
        .arg("--allow-duplicate-keys")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let golden = std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt")?;
    let expected = extract_block(
        &golden,
        "====[tsv-join --header -f input1.tsv -k 2 -a 5 --allow-duplicate-keys input2.tsv]====",
    );
    assert_eq!(stdout.trim_end(), expected.trim_end());

    Ok(())
}

#[test]
fn join_basic_allow_duplicate_keys_noheader_append_last_wins() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("5")
        .arg("--allow-duplicate-keys")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let golden = std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt")?;
    let expected = extract_block(
        &golden,
        "====[tsv-join -f input1_noheader.tsv -k 2 -a 5 --allow-duplicate-keys input2_noheader.tsv]====",
    );

    assert!(expected.starts_with("1\tggg\tUUU\t101b\t15b\t52"));
    assert_eq!(stdout.trim_end(), expected.trim_end());

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

#[test]
fn join_error_invalid_key_index_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("6")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: line has 5 fields, but key index 6 is out of range"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_append_index_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("6")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: line has 5 fields, but append index 6 is out of range"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_append_index_header_filter_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("6")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: line has 5 fields, but append index 6 is out of range"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_write_all_requires_append_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("--write-all")
        .arg("-1")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: --write-all requires --append-fields"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_write_all_cannot_be_used_with_exclude() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("--write-all")
        .arg("-1")
        .arg("--exclude")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: --write-all cannot be used with --exclude"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_missing_filter_file_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-k")
        .arg("2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("--filter-file") || stderr.contains("-f"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_missing_filter_file_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-k")
        .arg("2")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("--filter-file") || stderr.contains("-f"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_stdin_filter_without_data_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("-")
        .arg("-k")
        .arg("2")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: data file is required when filter-file is '-'"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_stdin_filter_without_data_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("-")
        .arg("-k")
        .arg("2")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: data file is required when filter-file is '-'"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2,0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_key_0_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("0,2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_data_0_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2,3")
        .arg("-d")
        .arg("0,2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_data_2_0() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2,3")
        .arg("-d")
        .arg("2,0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_2_0() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("1")
        .arg("-a")
        .arg("2,0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_0_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("1")
        .arg("-a")
        .arg("0,2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_f2_0() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("f2,0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_0_f2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("0,f2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_data_0_f2(
) -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("f2,f3")
        .arg("-d")
        .arg("0,f2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_data_f2_0(
) -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("f2,f3")
        .arg("-d")
        .arg("f2,0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_name_f2_0() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("f1")
        .arg("-a")
        .arg("f2,0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_name_0_f2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("f1")
        .arg("-a")
        .arg("0,f2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_header_name_key_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("no_field_6")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: unknown field name `no_field_6`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_header_name_append_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("4")
        .arg("-a")
        .arg("no_field_6")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: unknown field name `no_field_6`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_header_name_data_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("4")
        .arg("-d")
        .arg("no_field_6")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: unknown field name `no_field_6`")
            || stderr.contains("tva join: line has 1 fields, but key index 4 is out of range"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_different_number_of_keys_and_data_fields_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("-d")
        .arg("2,3")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: different number of key-fields and data-fields"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_different_number_of_keys_and_data_fields_header_name() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("f2")
        .arg("-d")
        .arg("f2,f3")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: different number of key-fields and data-fields"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_different_number_of_keys_and_data_fields_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("2")
        .arg("-d")
        .arg("2,3")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: different number of key-fields and data-fields"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_duplicate_keys_header_append_whole_line() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("0")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: field index must be >= 1 in `0`")
            || stderr.contains("tva join: duplicate key with different append values found in filter file"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_duplicate_keys_header_append_index() -> anyhow::Result<()> {
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

#[test]
fn join_error_duplicate_keys_noheader_append_whole_line() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("0")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: field index must be >= 1 in `0`")
            || stderr.contains("tva join: duplicate key with different append values found in filter file"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_duplicate_keys_noheader_append_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("2")
        .arg("-a")
        .arg("4")
        .arg("tests/data/join/input2_noheader.tsv")
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

#[test]
fn join_error_prefix_without_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--prefix")
        .arg("input1_")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: --prefix requires --header"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_exclude_with_append_fields_header_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("--exclude")
        .arg("-a")
        .arg("3")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("6")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: --exclude cannot be used with --append-fields"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_exclude_with_append_fields_noheader_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--exclude")
        .arg("-a")
        .arg("3")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("6")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: --exclude cannot be used with --append-fields"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_exclude_with_append_fields_header_name() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("--exclude")
        .arg("-a")
        .arg("f3")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("6")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: --exclude cannot be used with --append-fields"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_field_range_header_unknown_name_in_list() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2,x")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: unknown field name `x` in `2,x`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_field_range_noheader_name_requires_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("2,x")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: field name `x` requires header in `2,x`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_field_list_empty_element_noheader() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1_noheader.tsv")
        .arg("-k")
        .arg("2,,4")
        .arg("tests/data/join/input2_noheader.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: empty field list element in `2,,4`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_invalid_field_list_empty_element_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("--header")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("f2,,f4")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva join: empty field list element in `f2,,f4`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_no_such_filter_file() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/no_such-file.tsv")
        .arg("-k")
        .arg("2")
        .arg("tests/data/join/input2.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("could not open tests/data/join/no_such-file.tsv"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn join_error_no_such_data_file() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("join")
        .arg("-f")
        .arg("tests/data/join/input1.tsv")
        .arg("-k")
        .arg("2")
        .arg("tests/data/join/no_such-file.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("could not open tests/data/join/no_such-file.tsv"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}
