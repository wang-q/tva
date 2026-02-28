#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[test]
fn sample_shuffle_basic() {
    let input = "a\nb\nc\nd\n";
    let (stdout, _) = TvaCmd::new().args(&["sample"]).stdin(input).run();

    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.sort();
    assert_eq!(lines, vec!["a", "b", "c", "d"]);
}

#[test]
fn sample_num_limited() {
    let input = "a\nb\nc\nd\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "2"])
        .stdin(input)
        .run();
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }
}

#[test]
fn sample_prob_keeps_subset() {
    let input = "a\nb\nc\nd\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--prob", "0.5"])
        .stdin(input)
        .run();
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(lines.len() <= 4);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }
}

#[test]
fn sample_header_preserved() {
    let input = "h1\n1\n2\n3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--header", "--num", "2"])
        .stdin(input)
        .run();
    let mut lines = stdout.lines();

    let header = lines.next().unwrap();
    assert_eq!(header, "h1");

    let data: Vec<&str> = lines.collect();
    assert_eq!(data.len(), 2);
    for line in &data {
        assert!(["1", "2", "3"].contains(line));
    }
}

#[test]
fn sample_invalid_prob_rejected() {
    let (_, stderr) = TvaCmd::new().args(&["sample", "--prob", "0.0"]).run_fail();

    assert!(stderr.contains("invalid --prob/-p value"));
}

#[test]
fn sample_num_prob_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-n", "10", "-p", "0.5"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--num/-n and --prob/-p cannot be used together"));
}

#[test]
fn sample_replace_prob_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-r", "-p", "0.5"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--replace/-r cannot be used with --prob/-p"));
}

#[test]
fn sample_replace_no_num() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-r"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--replace/-r requires --num/-n greater than 0"));
}

#[test]
fn sample_inorder_conflicts() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-i", "-r", "-n", "5"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr
        .contains("--inorder/-i requires --num/-n without --replace/-r or --prob/-p"));
}

#[test]
fn sample_weight_prob_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-w", "1", "-p", "0.5"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--weight-field/-w cannot be used with --prob/-p"));
}

#[test]
fn sample_invalid_prob() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-p", "1.5"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("invalid --prob/-p value"));
}

#[test]
fn sample_gen_random_inorder_conflicts() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "-n", "10"])
        .stdin("a\n")
        .run_fail();

    assert!(
        stderr.contains("--gen-random-inorder cannot be combined with sampling options")
    );
}

#[test]
fn sample_weight_replace_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-w", "1", "-r", "-n", "10"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--weight-field/-w cannot be used with --replace/-r"));
}

#[test]
fn sample_key_no_prob() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-k", "1"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--key-fields/-k requires --prob/-p"));
}

#[test]
fn sample_key_conflicts() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-k", "1", "-p", "0.5", "-n", "10"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--key-fields/-k cannot be used with --num/-n"));
}

#[test]
fn sample_print_random_gen_random_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "--print-random", "--gen-random-inorder"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--print-random cannot be used with --gen-random-inorder"));
}

#[test]
fn sample_print_random_replace_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "--print-random", "-r", "-n", "10"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--print-random is not supported with --replace/-r"));
}

#[test]
fn sample_weight_index_out_of_range() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-w", "5", "-n", "1"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stderr.contains("weight field index 5 out of range"));
}

#[test]
fn sample_weight_invalid_value() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-w", "1", "-n", "1"])
        .stdin("not_a_number\n")
        .run_fail();

    assert!(stderr.contains("weight value `not_a_number` is not a valid number"));
}

#[test]
fn sample_key_index_out_of_range() {
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "-k", "5", "-p", "0.5"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stderr.contains("key field index 5 out of range"));
}

#[test]
fn sample_static_seed_produces_reproducible_output() {
    let input = "a\nb\nc\nd\n";

    let (s1, _) = TvaCmd::new()
        .args(&["sample", "--num", "3", "--static-seed"])
        .stdin(input)
        .run();

    let (s2, _) = TvaCmd::new()
        .args(&["sample", "--num", "3", "--static-seed"])
        .stdin(input)
        .run();

    assert_eq!(s1, s2);
}

#[test]
fn sample_replace_requires_num() {
    let input = "a\nb\nc\nd\n";
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "--replace"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("requires --num/-n greater than 0"));
}

#[test]
fn sample_replace_conflicts_with_prob() {
    let input = "a\nb\nc\nd\n";
    TvaCmd::new()
        .args(&["sample", "--replace", "--num", "2", "--prob", "0.5"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_replace_basic() {
    let input = "a\nb\nc\nd\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "10", "--replace", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }
}

#[test]
fn sample_inorder_requires_num() {
    let input = "a\nb\nc\nd\n";
    TvaCmd::new()
        .args(&["sample", "--inorder"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_inorder_conflicts_with_prob() {
    let input = "a\nb\nc\nd\n";
    TvaCmd::new()
        .args(&["sample", "--num", "2", "--prob", "0.5", "--inorder"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_inorder_conflicts_with_replace() {
    let input = "a\nb\nc\nd\n";
    TvaCmd::new()
        .args(&["sample", "--num", "2", "--replace", "--inorder"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_inorder_basic() {
    let input = "a\nb\nc\nd\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "2", "--inorder", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    let pos = |s: &str| match s {
        "a" => 0,
        "b" => 1,
        "c" => 2,
        "d" => 3,
        _ => 10,
    };
    assert!(pos(lines[0]) < pos(lines[1]));
}

#[test]
fn sample_weight_field_basic() {
    let input = "x\t1\nx\t10\nx\t100\nx\t1000\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "1",
            "--weight-field",
            "2",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "x\t1000");
}

#[test]
fn sample_weight_field_header_by_name() {
    let input = "name\tw\nx\t1\ny\t10\nz\t100\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--num",
            "1",
            "--weight-field",
            "w",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "name\tw");
}

#[test]
fn sample_weight_field_conflicts_with_prob_and_replace() {
    let input = "x\t1\nx\t10\n";

    TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "1",
            "--weight-field",
            "2",
            "--prob",
            "0.5",
        ])
        .stdin(input)
        .run_fail();

    TvaCmd::new()
        .args(&["sample", "--num", "1", "--weight-field", "2", "--replace"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_weight_field_invalid_field_list_reports_error() {
    let input = "x\t1\nx\t10\n";
    let (_, stderr) = TvaCmd::new()
        .args(&["sample", "--num", "1", "--weight-field", "0"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("tva sample:"));
}

#[test]
fn sample_key_fields_requires_prob() {
    let input = "k\tv\na\t1\n";
    TvaCmd::new()
        .args(&["sample", "--header", "--key-fields", "k"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_key_fields_distinct_per_key() {
    let input = "k\tv\na\t1\na\t2\nb\t3\nb\t4\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--prob",
            "0.5",
            "--key-fields",
            "k",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let mut lines = stdout.lines();

    let header = lines.next().unwrap();
    assert_eq!(header, "k\tv");

    let data: Vec<&str> = lines.collect();
    let mut count_a = 0;
    let mut count_b = 0;
    for line in &data {
        let cols: Vec<&str> = line.split('\t').collect();
        match cols[0] {
            "a" => count_a += 1,
            "b" => count_b += 1,
            _ => {}
        }
    }

    assert!(count_a == 0 || count_a == 2);
    assert!(count_b == 0 || count_b == 2);
}

#[test]
fn sample_gen_random_inorder_basic() {
    let input = "k\tv\na\t1\nb\t2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--gen-random-inorder",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let mut lines = stdout.lines();

    let header = lines.next().unwrap();
    let header_cols: Vec<&str> = header.split('\t').collect();
    assert_eq!(header_cols[0], "random_value");
    assert_eq!(header_cols[1], "k");
    assert_eq!(header_cols[2], "v");

    let data: Vec<&str> = lines.collect();
    assert_eq!(data.len(), 2);

    let cols1: Vec<&str> = data[0].split('\t').collect();
    assert!(cols1[0].parse::<f64>().is_ok());
    assert_eq!(cols1[1], "a");
    assert_eq!(cols1[2], "1");

    let cols2: Vec<&str> = data[1].split('\t').collect();
    assert!(cols2[0].parse::<f64>().is_ok());
    assert_eq!(cols2[1], "b");
    assert_eq!(cols2[2], "2");
}

#[test]
fn sample_gen_random_inorder_conflicts_with_sampling() {
    let input = "a\nb\nc\n";

    TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "--num", "2"])
        .stdin(input)
        .run_fail();

    TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "--prob", "0.5"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_print_random_basic() {
    let input = "a\nb\nc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--print-random", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    for line in &lines {
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols[0].parse::<f64>().is_ok());
    }
}

#[test]
fn sample_print_random_not_allowed_with_replace() {
    let input = "a\nb\nc\n";
    TvaCmd::new()
        .args(&["sample", "--num", "5", "--replace", "--print-random"])
        .stdin(input)
        .run_fail();
}

#[test]
fn sample_compat_num_superset() {
    let mut input = String::new();
    for i in 0..20 {
        input.push_str(&format!("{}\n", i));
    }

    let (stdout_small, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--compatibility-mode",
            "--static-seed",
            "--num",
            "5",
        ])
        .stdin(input.clone())
        .run();
    let lines_small: HashSet<String> =
        stdout_small.lines().map(|s| s.to_string()).collect();

    let (stdout_large, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--compatibility-mode",
            "--static-seed",
            "--num",
            "10",
        ])
        .stdin(input)
        .run();
    let lines_large: HashSet<String> =
        stdout_large.lines().map(|s| s.to_string()).collect();

    assert!(lines_small.is_subset(&lines_large));
}

#[test]
fn sample_compat_multi_file_from_tsv_sample_inputs() {
    let base = PathBuf::from("tests/data/sample");
    let input1 = base.join("input3x10.tsv");
    let input2 = base.join("input3x25.tsv");

    let header_input = fs::read_to_string(&input1).unwrap();
    let mut header_lines = header_input.lines();
    let expected_header = header_lines.next().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--static-seed",
            "--compatibility-mode",
            input1.to_str().unwrap(),
            input2.to_str().unwrap(),
        ])
        .run();

    let mut out_lines = stdout.lines();
    let header = out_lines.next().unwrap();
    assert_eq!(header, expected_header);

    fn count_data_rows(path: &PathBuf) -> anyhow::Result<usize> {
        let contents = fs::read_to_string(path)?;
        let mut it = contents.lines();
        let _ = it.next();
        Ok(it.count())
    }

    let expected_rows =
        count_data_rows(&input1).unwrap() + count_data_rows(&input2).unwrap();
    let out_data: Vec<&str> = out_lines.collect();
    assert_eq!(out_data.len(), expected_rows);
}

#[test]
fn sample_compat_stdin_and_files_from_tsv_sample_inputs() {
    let base = PathBuf::from("tests/data/sample");
    let stdin_path = base.join("input3x10.tsv");
    let file1 = base.join("input3x3.tsv");
    let file2 = base.join("input3x4.tsv");

    let stdin_data = fs::read_to_string(&stdin_path).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--static-seed",
            "--compatibility-mode",
            "--",
            "-",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .stdin(stdin_data)
        .run();

    let mut out_lines = stdout.lines();
    let header = out_lines.next().unwrap();

    let stdin_header = fs::read_to_string(&stdin_path).unwrap();
    let mut stdin_lines = stdin_header.lines();
    let expected_header = stdin_lines.next().unwrap();
    assert_eq!(header, expected_header);

    fn count_rows_with_header(
        path: &PathBuf,
        has_header: bool,
    ) -> anyhow::Result<usize> {
        let contents = fs::read_to_string(path)?;
        let mut it = contents.lines();
        if has_header {
            let _ = it.next();
        }
        Ok(it.count())
    }

    let expected_rows = count_rows_with_header(&stdin_path, true).unwrap()
        + count_rows_with_header(&file1, true).unwrap()
        + count_rows_with_header(&file2, true).unwrap();

    let out_data: Vec<&str> = out_lines.collect();
    assert_eq!(out_data.len(), expected_rows);
}

#[test]
fn sample_windows_newlines_from_tsv_sample_inputs() {
    let base = PathBuf::from("tests/data/sample");
    let unix_path = base.join("input3x25.tsv");
    let dos_path = base.join("input3x25.dos_tsv");

    let unix_contents = fs::read_to_string(&unix_path).unwrap();
    let mut unix_lines = unix_contents.lines();
    let unix_header = unix_lines.next().unwrap();
    let unix_data_count = unix_lines.count();

    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--header", dos_path.to_str().unwrap()])
        .run();

    let mut out_lines = stdout.lines();
    let header = out_lines.next().unwrap();
    assert_eq!(header, unix_header);

    let out_data: Vec<&str> = out_lines.collect();
    assert_eq!(out_data.len(), unix_data_count);
}
