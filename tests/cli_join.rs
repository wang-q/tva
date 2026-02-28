#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

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
fn join_basic_inner_join_header_by_name() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-H",
            "--filter-file",
            "tests/data/join/filter_basic.tsv",
            "--key-fields",
            "id",
            "--append-fields",
            "fv1,fv2",
            "tests/data/join/data_basic.tsv",
        ])
        .run();

    assert_eq!(
        stdout,
        "id\tdv1\tfv1\tfv2\nk1\tx1\tv1a\tv1b\nk2\tx2\tv2a\tv2b\n"
    );
}

#[test]
fn join_basic_from_golden_whole_line_key_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "--filter-file",
            "tests/data/join/input1.tsv",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let golden =
        std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt").unwrap();
    let expected = extract_block(
        &golden,
        "====[tsv-join --header --filter-file input1.tsv input2.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());
}

#[test]
fn join_basic_line_buffered_header_filter_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--line-buffered",
            "--header",
            "--filter-file",
            "tests/data/join/input1.tsv",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let golden =
        std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt").unwrap();
    let expected = extract_block(
        &golden,
        "====[tsv-join --header --filter-file input1.tsv input2.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());
}

#[test]
fn join_basic_line_buffered_noheader_filter_file() {
    let (stdout1, _) = TvaCmd::new()
        .args(&[
            "join",
            "--filter-file",
            "tests/data/join/input1_noheader.tsv",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&[
            "join",
            "--line-buffered",
            "--filter-file",
            "tests/data/join/input1_noheader.tsv",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run();

    assert_eq!(stdout1, stdout2);
}

#[test]
fn join_basic_line_buffered_header_key_fields_1() {
    let (stdout1, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "--key-fields",
            "1",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&[
            "join",
            "--line-buffered",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "--key-fields",
            "1",
            "tests/data/join/input2.tsv",
        ])
        .run();

    assert_eq!(stdout1, stdout2);
}

#[test]
fn join_basic_line_buffered_noheader_key_fields_1() {
    let (stdout1, _) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "--key-fields",
            "1",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&[
            "join",
            "--line-buffered",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "--key-fields",
            "1",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run();

    assert_eq!(stdout1, stdout2);
}

#[test]
fn join_basic_line_buffered_header_data_fields() {
    let (stdout1, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "--data-fields",
            "2",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&[
            "join",
            "--line-buffered",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "--data-fields",
            "2",
            "tests/data/join/input2.tsv",
        ])
        .run();

    assert_eq!(stdout1, stdout2);
}

#[test]
fn join_basic_line_buffered_header_allow_duplicate_keys_append() {
    let (stdout1, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-a",
            "5",
            "--allow-duplicate-keys",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&[
            "join",
            "--line-buffered",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-a",
            "5",
            "--allow-duplicate-keys",
            "tests/data/join/input2.tsv",
        ])
        .run();

    assert_eq!(stdout1, stdout2);
}

#[test]
fn join_basic_line_buffered_header_allow_duplicate_keys_append_whole_line() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--line-buffered",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "3",
            "-d",
            "2",
            "-a",
            "0",
            "--allow-duplicate-keys",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: field index must be >= 1 in `0`"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_basic_inner_join_header_by_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-H",
            "--filter-file",
            "tests/data/join/filter_basic.tsv",
            "-k",
            "1",
            "-a",
            "2,3",
            "tests/data/join/data_basic.tsv",
        ])
        .run();

    assert_eq!(
        stdout,
        "id\tdv1\tfv1\tfv2\nk1\tx1\tv1a\tv1b\nk2\tx2\tv2a\tv2b\n"
    );
}

#[test]
fn join_basic_write_all_left_outer_join_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "--allow-duplicate-keys",
            "-k",
            "2",
            "-a",
            "5",
            "--write-all",
            "NA",
            "tests/data/join/input2.tsv",
        ])
        .run();

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
}

#[test]
fn join_basic_write_all_multi_append_fields_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "--allow-duplicate-keys",
            "-k",
            "2",
            "-a",
            "4,5",
            "--write-all",
            "NA",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "f1\tf2\tf3\tf4\tf5\tf4\tf5");

    assert!(stdout.contains("27\txa\tgg\t44\t45\tNA\tNA"));
}

#[test]
fn join_basic_from_golden_whole_line_key_header_short_opts() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "--key-fields",
            "0",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let golden =
        std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt").unwrap();
    let expected = extract_block(
        &golden,
        "====[tsv-join --header -f input1.tsv --key-fields 0 input2.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());
}

#[test]
fn join_basic_write_all_left_outer_join_noheader() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "--allow-duplicate-keys",
            "-k",
            "2",
            "-a",
            "5",
            "--write-all",
            "NA",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run();

    assert!(stdout.contains("27\txa\tgg\t44\t45\tNA"));
}

#[test]
fn join_basic_single_column_filter_header_key_only() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input_1x5.tsv",
            "-k",
            "1",
            "tests/data/join/input1.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines[0], "f1\tf2\tf3\tf4\tf5");
    assert_eq!(lines.len(), 2);
    assert!(lines[1].starts_with("3\tnnn\tGGG\t336"));
}

#[test]
fn join_basic_single_column_filter_header_data_and_append() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input_1x5.tsv",
            "-k",
            "1",
            "-d",
            "2",
            "-a",
            "1",
            "tests/data/join/input1.tsv",
        ])
        .run();

    let expected = concat!(
        "f1\tf2\tf3\tf4\tf5\tfa\n",
        "1\tggg\tUUU\t101\t15\tggg\n",
        "5\tggg\tCCC\t5734\t52\tggg\n",
        "9\tv\tvv\t97\t91\tv\n",
    );

    assert_eq!(stdout, expected);
}

#[test]
fn join_basic_alternate_delimiter_header_key_only() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--delimiter",
            ":",
            "--header",
            "-k",
            "1",
            "-f",
            "tests/data/join/input_2x3_colon.tsv",
            "tests/data/join/input_5x4_colon.tsv",
        ])
        .run();

    let expected = concat!(
        "Field A:Field B:Field C:Field D:Field E\n",
        "13:hello world:fast:432:303\n",
        "101:501:432:12:13\n",
    );

    assert_eq!(stdout, expected);
}

#[test]
fn join_basic_alternate_delimiter_header_with_data_fields() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--delimiter",
            ":",
            "--header",
            "-k",
            "1",
            "-d",
            "5",
            "-f",
            "tests/data/join/input_2x3_colon.tsv",
            "tests/data/join/input_5x4_colon.tsv",
        ])
        .run();

    let expected = concat!(
        "Field A:Field B:Field C:Field D:Field E\n",
        "55:\tabc:501:892:101\n",
        "101:501:432:12:13\n",
    );

    assert_eq!(stdout, expected);
}

#[test]
fn join_basic_alternate_delimiter_header_multi_key_append() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--delimiter",
            ":",
            "--header",
            "-k",
            "1,2",
            "-d",
            "5,3",
            "-a",
            "1",
            "-f",
            "tests/data/join/input_2x3_colon.tsv",
            "tests/data/join/input_5x4_colon.tsv",
        ])
        .run();

    assert_eq!(
        stdout.trim_end(),
        "Field A:Field B:Field C:Field D:Field E:col a"
    );
}

#[test]
fn join_basic_alternate_delimiter_header_named_fields() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--delimiter",
            ":",
            "--header",
            "-d",
            "col a,col b",
            "-k",
            "Field E,Field C",
            "-a",
            "Field B",
            "-f",
            "tests/data/join/input_5x4_colon.tsv",
            "tests/data/join/input_2x3_colon.tsv",
        ])
        .run();

    assert_eq!(stdout.trim_end(), "col a:col b:Field B");
}

#[test]
fn join_basic_empty_filter_noheader() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input_emptyfile.tsv",
            "tests/data/join/input2.tsv",
        ])
        .run();

    assert!(stdout.is_empty());
}

#[test]
fn join_basic_empty_filter_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-H",
            "-f",
            "tests/data/join/input_emptyfile.tsv",
            "tests/data/join/input2.tsv",
        ])
        .run();

    assert_eq!(stdout, "f1\tf2\tf3\tf4\tf5\n");
}

#[test]
fn join_basic_empty_data_noheader() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1.tsv",
            "tests/data/join/input_emptyfile.tsv",
        ])
        .run();

    assert!(stdout.is_empty());
}

#[test]
fn join_basic_empty_data_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-H",
            "-f",
            "tests/data/join/input1.tsv",
            "tests/data/join/input_emptyfile.tsv",
        ])
        .run();

    assert!(stdout.is_empty());
}

#[test]
fn join_error_delimiter_must_be_single_char() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--filter-file",
            "tests/data/join/filter_basic.tsv",
            "--delimiter",
            "::",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: delimiter must be a single character"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_basic_exclude_header_whole_line_key() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "--exclude",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let golden =
        std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt").unwrap();
    let expected = extract_block(
        &golden,
        "====[tsv-join --header -f input1.tsv --exclude input2.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());
}

#[test]
fn join_basic_exclude_noheader_whole_line_key() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "--exclude",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run();

    let golden =
        std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt").unwrap();
    let expected = extract_block(
        &golden,
        "====[tsv-join -f input1_noheader.tsv --exclude input2_noheader.tsv]====",
    );

    assert_eq!(stdout.trim_end(), expected.trim_end());
}

#[test]
fn join_basic_allow_duplicate_keys_header_append_last_wins() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-a",
            "5",
            "--allow-duplicate-keys",
            "tests/data/join/input2.tsv",
        ])
        .run();

    let golden =
        std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt").unwrap();
    let expected = extract_block(
        &golden,
        "====[tsv-join --header -f input1.tsv -k 2 -a 5 --allow-duplicate-keys input2.tsv]====",
    );
    assert_eq!(stdout.trim_end(), expected.trim_end());
}

#[test]
fn join_basic_allow_duplicate_keys_noheader_append_last_wins() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "2",
            "-a",
            "5",
            "--allow-duplicate-keys",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run();

    let golden =
        std::fs::read_to_string("tests/data/join/gold_basic_tests_1.txt").unwrap();
    let expected = extract_block(
        &golden,
        "====[tsv-join -f input1_noheader.tsv -k 2 -a 5 --allow-duplicate-keys input2_noheader.tsv]====",
    );

    assert!(expected.starts_with("1\tggg\tUUU\t101b\t15b\t52"));
    assert_eq!(stdout.trim_end(), expected.trim_end());
}

#[test]
fn join_error_duplicate_keys_filter_header_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-a",
            "4",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains(
            "tva join: duplicate key with different append values found in filter file"
        ),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_key_index_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "6",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: line has 5 fields, but key index 6 is out of range"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_append_index_noheader() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "2",
            "-a",
            "6",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run_fail();

    assert!(
        stderr
            .contains("tva join: line has 5 fields, but append index 6 is out of range"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_append_index_header_filter_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-a",
            "6",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr
            .contains("tva join: line has 5 fields, but append index 6 is out of range"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_write_all_requires_append_fields() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "--write-all",
            "-1",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: --write-all requires --append-fields"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_write_all_cannot_be_used_with_exclude() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "--write-all",
            "-1",
            "--exclude",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: --write-all cannot be used with --exclude"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_missing_filter_file_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["join", "--header", "-k", "2", "tests/data/join/input2.tsv"])
        .run_fail();

    assert!(
        stderr.contains("--filter-file") || stderr.contains("-f"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_missing_filter_file_noheader() {
    let (_, stderr) = TvaCmd::new()
        .args(&["join", "-k", "2", "tests/data/join/input2_noheader.tsv"])
        .run_fail();

    assert!(
        stderr.contains("--filter-file") || stderr.contains("-f"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_stdin_filter_without_data_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["join", "--header", "-f", "-", "-k", "2"])
        .run_fail();

    assert!(
        stderr.contains("tva join: data file is required when filter-file is '-'"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_stdin_filter_without_data_noheader() {
    let (_, stderr) = TvaCmd::new()
        .args(&["join", "-f", "-", "-k", "2"])
        .run_fail();

    assert!(
        stderr.contains("tva join: data file is required when filter-file is '-'"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2,0",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_key_0_2() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "0,2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_data_0_2() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2,3",
            "-d",
            "0,2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_data_2_0() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2,3",
            "-d",
            "2,0",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_2_0() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "1",
            "-a",
            "2,0",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_0_2() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "1",
            "-a",
            "0,2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_f2_0() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "f2,0",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_0_f2() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "0,f2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_data_0_f2() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "f2,f3",
            "-d",
            "0,f2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_key_and_fields_header_name_data_f2_0() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "f2,f3",
            "-d",
            "f2,0",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_name_f2_0() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "f1",
            "-a",
            "f2,0",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_whole_line_combo_append_header_name_0_f2() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "f1",
            "-a",
            "0,f2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("field index must be >= 1")
            || stderr.contains("Field 0 (whole line) cannot be combined"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_header_name_key_fields() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "no_field_6",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: unknown field name `no_field_6`"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_header_name_append_fields() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "4",
            "-a",
            "no_field_6",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: unknown field name `no_field_6`"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_header_name_data_fields() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "4",
            "-d",
            "no_field_6",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: unknown field name `no_field_6`")
            || stderr.contains(
                "tva join: line has 1 fields, but key index 4 is out of range"
            ),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_different_number_of_keys_and_data_fields_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-d",
            "2,3",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: different number of key-fields and data-fields"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_different_number_of_keys_and_data_fields_header_name() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "f2",
            "-d",
            "f2,f3",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: different number of key-fields and data-fields"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_different_number_of_keys_and_data_fields_noheader() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "2",
            "-d",
            "2,3",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: different number of key-fields and data-fields"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_duplicate_keys_header_append_whole_line() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-a",
            "0",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: field index must be >= 1 in `0`")
            || stderr.contains("tva join: duplicate key with different append values found in filter file"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_duplicate_keys_header_append_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "-a",
            "4",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains(
            "tva join: duplicate key with different append values found in filter file"
        ),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_duplicate_keys_noheader_append_whole_line() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "2",
            "-a",
            "0",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: field index must be >= 1 in `0`")
            || stderr.contains("tva join: duplicate key with different append values found in filter file"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_duplicate_keys_noheader_append_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "2",
            "-a",
            "4",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains(
            "tva join: duplicate key with different append values found in filter file"
        ),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_prefix_without_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--prefix",
            "input1_",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: --prefix requires --header"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_exclude_with_append_fields_header_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "--exclude",
            "-a",
            "3",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "6",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: --exclude cannot be used with --append-fields"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_exclude_with_append_fields_noheader_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--exclude",
            "-a",
            "3",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "6",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: --exclude cannot be used with --append-fields"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_exclude_with_append_fields_header_name() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "--exclude",
            "-a",
            "f3",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "6",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: --exclude cannot be used with --append-fields"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_field_range_header_unknown_name_in_list() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2,x",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: unknown field name `x` in `2,x`"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_field_range_noheader_name_requires_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "2,x",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: field name `x` requires header in `2,x`"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_field_list_empty_element_noheader() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1_noheader.tsv",
            "-k",
            "2,,4",
            "tests/data/join/input2_noheader.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: empty field list element in `2,,4`"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_invalid_field_list_empty_element_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "--header",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "f2,,f4",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("tva join: empty field list element in `f2,,f4`"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_no_such_filter_file() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/no_such-file.tsv",
            "-k",
            "2",
            "tests/data/join/input2.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("could not open tests/data/join/no_such-file.tsv"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn join_error_no_such_data_file() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "join",
            "-f",
            "tests/data/join/input1.tsv",
            "-k",
            "2",
            "tests/data/join/no_such-file.tsv",
        ])
        .run_fail();

    assert!(
        stderr.contains("could not open tests/data/join/no_such-file.tsv"),
        "stderr was: {}",
        stderr
    );
}
