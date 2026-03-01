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

#[test]
fn sample_distinct_basic() {
    // 50% probability, distinct by key (field 1).
    // a appears 3 times. b appears 2 times.
    // Result should have either ALL 'a' lines or NO 'a' lines.
    // Same for 'b'.
    let input = "a	1
a	2
b	1
a	3
b	2
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "1",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    let a_count = lines.iter().filter(|l| l.starts_with("a	")).count();
    let b_count = lines.iter().filter(|l| l.starts_with("b	")).count();

    // With static seed, one of them might be selected.
    // Critically, a_count must be 0 or 3. b_count must be 0 or 2.
    assert!(a_count == 0 || a_count == 3, "a_count was {}", a_count);
    assert!(b_count == 0 || b_count == 2, "b_count was {}", b_count);
}

#[test]
fn sample_weighted_shuffle() {
    // Weighted shuffle (no --num).
    // High weight items should appear earlier on average.
    // For a deterministic test, we check that all items are present.
    let input = "A	1
B	100
C	1
";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--weight-field", "2", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.contains(&"A	1"));
    assert!(lines.contains(&"B	100"));
    assert!(lines.contains(&"C	1"));

    // With B having high weight, it likely comes first.
    // checking order with static seed might be fragile if algorithm changes,
    // but useful for regression.
    // For now just check content.
}

#[test]
fn sample_print_random() {
    let input = "a\nb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "1", "--print-random", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    // Format: random_value\tline
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 2);
    // Verify first part is float
    let _val: f64 = parts[0]
        .parse()
        .expect("First field should be random value");
}

#[test]
fn sample_gen_random_inorder() {
    let input = "a
b
c
";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    // Order must be preserved
    assert!(lines[0].ends_with("\ta"));
    assert!(lines[1].ends_with("\tb"));
    assert!(lines[2].ends_with("\tc"));

    // Check headers if we add --header
    let input_h = "h\na\nb\n";
    let (stdout_h, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--header",
            "--static-seed",
        ])
        .stdin(input_h)
        .run();

    let lines_h: Vec<&str> = stdout_h.lines().collect();
    assert_eq!(lines_h[0], "random_value\th");
    assert!(lines_h[1].ends_with("\ta"));
}

#[test]
fn sample_multiple_files() {
    use std::fs::File;
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let file1_path = dir.path().join("file1.tsv");
    let file2_path = dir.path().join("file2.tsv");

    {
        let mut f1 = File::create(&file1_path).unwrap();
        writeln!(f1, "f1_a").unwrap();
        writeln!(f1, "f1_b").unwrap();

        let mut f2 = File::create(&file2_path).unwrap();
        writeln!(f2, "f2_a").unwrap();
        writeln!(f2, "f2_b").unwrap();
    }

    // Shuffle all
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines.contains(&"f1_a"));
    assert!(lines.contains(&"f1_b"));
    assert!(lines.contains(&"f2_a"));
    assert!(lines.contains(&"f2_b"));
}

#[test]
fn sample_distinct_with_header() {
    let input = "k	v
A	1
A	2
B	1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "k",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0] == "k	v");

    // Data lines
    let data = &lines[1..];
    let a_count = data.iter().filter(|l| l.starts_with("A	")).count();
    assert!(a_count == 0 || a_count == 2);
}

#[test]
fn sample_distinct_k0_vs_all_fields() {
    // -k 0 (whole line) vs -k * (all fields)
    // Data:
    // A	1
    // A	1 (duplicate)
    // A	2 (distinct)
    // B	1
    let input = "A	1
A	1
A	2
B	1
";

    // -k 0
    let (stdout_k0, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "0",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    // -k * (requires header for wildcard expansion usually, but let's check if * works without header?
    // fields.rs says parse_field_list_with_header requires header for wildcard.
    // If no header, * is treated as literal or error?
    // tsv-sample.d uses fpath_data2x25 which HAS header for -k *.
    // Let's use header for -k * test.
    let input_header = "f1	f2
A	1
A	1
A	2
B	1
";

    let (stdout_k_star, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "*",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input_header)
        .run();

    // Check content.
    // A	1 appears twice. Distinct sampling should treat them as ONE key.
    // If selected, BOTH should appear. If not, NEITHER.
    // A	2 is different key.
    // B	1 is different key.

    let lines_k0: Vec<&str> = stdout_k0.lines().collect();
    let a1_count = lines_k0.iter().filter(|l| **l == "A	1").count();
    assert!(a1_count == 0 || a1_count == 2);

    let lines_star: Vec<&str> = stdout_k_star.lines().collect();
    // skip header
    let data_star = &lines_star[1..];
    let a1_count_star = data_star.iter().filter(|l| **l == "A	1").count();
    assert!(a1_count_star == 0 || a1_count_star == 2);
}

#[test]
fn sample_distinct_no_header() {
    // No header, -k 1
    let input = "A	1
A	2
B	1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "1",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let a_count = lines.iter().filter(|l| l.starts_with("A	")).count();
    // A is key. 2 rows have key A.
    assert!(a_count == 0 || a_count == 2);
}

#[test]
fn sample_distinct_print_random() {
    let input = "A	1
A	1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "0",
            "--prob",
            "0.5",
            "--print-random",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    if !lines.is_empty() {
        // If selected, both lines should be present and have SAME random value?
        // tsv-sample distinct sampling assigns random value per KEY.
        // So they should be identical.
        let parts0: Vec<&str> = lines[0].split('\t').collect();
        let parts1: Vec<&str> = lines[1].split('\t').collect();
        assert_eq!(parts0[0], parts1[0]); // Random values equal
        assert_eq!(
            lines[0].split('\t').skip(1).collect::<Vec<_>>(),
            vec!["A", "1"]
        );
    }
}

#[test]
fn sample_distinct_gen_random_inorder() {
    let input = "A\t1\nA\t1\nB\t1\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "0",
            "--gen-random-inorder",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    let parts0: Vec<&str> = lines[0].split('\t').collect();
    let parts1: Vec<&str> = lines[1].split('\t').collect();
    let parts2: Vec<&str> = lines[2].split('\t').collect();

    // A	1 and A	1 should have same random value
    assert_eq!(parts0[0], parts1[0]);
    // B	1 should likely have different value
    // (technically possible to be same collision, but unlikely with double)
    assert!(parts0[0].parse::<f64>().is_ok());

    assert_eq!(parts0[1], "A");
    assert_eq!(parts0[2], "1");

    assert!(parts2[0].parse::<f64>().is_ok());
    assert_eq!(parts2[1], "B");
    assert_eq!(parts2[2], "1");
}

#[test]
fn sample_distinct_multiple_keys() {
    // -k 1,2
    let input = "A	1	X
A	1	Y
A	2	X
";
    // Keys: (A,1), (A,1), (A,2)
    // Row 1 and 2 share key (A,1) and should be treated as duplicates for sampling.

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "1,2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let a1_count = lines.iter().filter(|l| l.starts_with("A	1")).count();
    assert!(a1_count == 0 || a1_count == 2);

    let a2_count = lines.iter().filter(|l| l.starts_with("A	2")).count();
    assert!(a2_count == 0 || a2_count == 1);
}

#[test]
fn sample_weight_field_header_by_name_multiple_cols() {
    let input = "h1\th2\th3\nA\t1\t100\nB\t2\t1\n";
    // Weight field h3 (100 vs 1). A should likely be first if shuffled, but here we just check it runs.
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--weight-field",
            "h3",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3); // Header + 2 rows
    assert_eq!(lines[0], "h1	h2	h3");
    // Check data presence
    assert!(lines.contains(&"A	1	100"));
    assert!(lines.contains(&"B	2	1"));
}

#[test]
fn sample_weight_field_header_by_wildcard() {
    let input = "h1	h2	weight_col
A	1	100
B	2	1
";
    // Test --weight-field with wildcard matching (e.g., "weight_*").

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--weight-field",
            "weight_*",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn sample_inorder_numeric() {
    // --inorder with --num
    // Input is sorted 0..9. Output should be a subset but still sorted.
    let input: String = (0..10)
        .map(|i| format!("line{}", i))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "5", "--inorder", "--static-seed"])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 5);

    // Check order
    let indices: Vec<usize> = lines
        .iter()
        .map(|l| l.replace("line", "").parse::<usize>().unwrap())
        .collect();

    for i in 0..indices.len() - 1 {
        assert!(
            indices[i] < indices[i + 1],
            "Output not in order: {:?}",
            indices
        );
    }
}

#[test]
fn sample_replace_multiple_items() {
    // --replace --num 10 from 3 items.
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--replace", "--num", "10", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10);
    // Should contain duplicates
    let unique: std::collections::HashSet<&str> = lines.iter().cloned().collect();
    assert!(unique.len() <= 3);
}

#[test]
fn sample_reservoir_algo_r() {
    // --prefer-algorithm-r --num 2 from 5
    let input = "1
2
3
4
5
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--prefer-algorithm-r",
            "--num",
            "2",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    // Lines should be unique
    assert_ne!(lines[0], lines[1]);
}

#[test]
fn sample_bernoulli_skip() {
    // Low probability to trigger skip sampling (if implemented or standard).
    // p=0.01 on 1000 lines -> expect approx 10 lines.
    let input: String = (0..1000)
        .map(|i| format!("line{}", i))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--prob", "0.01", "--static-seed"])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // It's random, but with static seed it's deterministic.
    assert!(!lines.is_empty());
    assert!(lines.len() < 200);
}

#[test]
fn sample_subset_num_greater_than_input() {
    // -n 10 on 3 lines. Should return 3 lines (shuffled).
    let input = "A
B
C
";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "10", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.contains(&"A"));
    assert!(lines.contains(&"B"));
    assert!(lines.contains(&"C"));
}

#[test]
fn sample_distinct_key_ranges() {
    // -k 1-2
    let input = "A	1	X
A	1	Y
B	2	Z
";
    // (A,1) duplicated.
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "1-2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Count A	1
    let a1_count = lines.iter().filter(|l| l.starts_with("A	1")).count();
    assert!(a1_count == 0 || a1_count == 2);
}

#[test]
fn sample_distinct_key_names_range() {
    // -k h1-h2
    let input = "h1	h2	h3
A	1	X
A	1	Y
B	2	Z
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "h1-h2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Count A	1
    let a1_count = lines.iter().filter(|l| l.starts_with("A	1")).count();
    assert!(a1_count == 0 || a1_count == 2);
}

#[test]
fn sample_print_random_with_header_custom() {
    let input = "h1
A
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--prob",
            "1.0",
            "--print-random",
            "--random-value-header",
            "RVAL",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "RVAL	h1");
    assert!(lines[1].ends_with("	A"));
}

#[test]
fn sample_distinct_key_names_list() {
    // -k h1,h2
    let input = "h1	h2	h3
A	1	X
A	1	Y
B	2	Z
";
    // Key is (h1, h2).
    // Rows 1 and 2 are (A,1). Row 3 is (B,2).
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "h1,h2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Count A	1
    let a1_count = lines.iter().filter(|l| l.starts_with("A	1")).count();
    assert!(a1_count == 0 || a1_count == 2);
}

#[test]
fn sample_weighted_inorder() {
    // Weighted sampling with --inorder.
    // Weights: A=10, B=1, C=100.
    // Select 2. Likely A and C.
    // If inorder, output should be A then C.
    // If not inorder, WeightedReservoirSampler currently sorts by Key descending (highest weight key first).
    // Key = ln(u) / w. Large w -> small key? No.
    // u in (0,1). ln(u) is negative.
    // A-Res key: k = u^(1/w).
    // tva implementation uses: key = u.ln() / w.
    // key is negative.
    // Larger w -> smaller abs(key) -> larger key (closer to 0).
    // So large weights have larger keys.
    // Implementation sorts by key descending. So largest key (largest weight) comes first.
    // So C should come before A without --inorder.

    let input = "A	10
B	1
C	100
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "2",
            "--weight-field",
            "2",
            "--inorder",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);

    // Check if output is sorted by original order (A appears before C)
    // We assume A and C are selected due to high weights and static seed (or probability).
    // B has very low weight.

    let pos_a = lines.iter().position(|&r| r.starts_with("A"));
    let pos_c = lines.iter().position(|&r| r.starts_with("C"));

    if let (Some(pa), Some(pc)) = (pos_a, pos_c) {
        assert!(pa < pc, "Output was not inorder: {:?}", lines);
    }
}

#[test]
fn sample_compat_mode_single_col() {
    // --compatibility-mode
    // Usually uses Mersenne Twister. tva uses RapidRng.
    // Compatibility mode in tva primarily affects the algorithm choice (e.g. CompatRandomSampler).
    // It reads all lines into memory and shuffles.
    let input: String = (0..10)
        .map(|i| format!("line{}", i))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--compatibility-mode",
            "--header",
            "--static-seed",
        ])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Should contain all lines (no -n specified, shuffle mode)
    // 10 lines + header? No, input has no header really, but we said --header.
    // So line0 is header.
    // Remaining 9 lines shuffled.
    assert_eq!(lines.len(), 10);
    assert_eq!(lines[0], "line0");

    // Check if other lines are present
    for i in 1..10 {
        assert!(lines.contains(&format!("line{}", i).as_str()));
    }
}

#[test]
fn sample_weighted_print_random() {
    // -w 2 --print-random
    // Should output: random_value\toriginal_line
    // tva implementation:
    // finalize calls write_with_optional_random(..., None).
    // write_with_optional_random generates rng.next().
    // This is NOT the key used for selection!
    // This seems to be a discrepancy.
    // If user asks for --print-random with weighted sampling, they probably want to see the weight/key?
    // Or just a random number?
    // tsv-sample docs say "Print the random value used for sampling".
    // For weighted reservoir (A-Res), that value is the Key.

    // We should fix this in WeightedReservoirSampler to store the key and output it if print_random is true.

    let input = "h1\th2\nA\t10\nB\t1\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--weight-field",
            "2",
            "--print-random",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Check header
    assert!(lines[0].starts_with("random_value\th1"));

    // Check values.
    // A has weight 10. Key = ln(u1)/10. (Negative, close to 0)
    // B has weight 1. Key = ln(u2)/1. (Negative, can be large magnitude)
    // Output should be float\tline.
    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert!(parts[0].parse::<f64>().is_ok());
}

#[test]
fn sample_key_fields_star() {
    let input = "h1	h2
A	1
B	1
A	2
";
    // Use header for -k * test.
    // k=* means all fields are keys.
    // Keys: (A,1), (B,1), (A,2). All distinct.
    // prob 0.5 means roughly half are kept.
    // But since they are all distinct keys, it's just distinct sampling.
    // Distinct sampling with prob 0.5 means each key has 0.5 chance.
    // With 3 items, we expect 0-3 items.
    // With static seed, we might get fewer than 3.
    // Let's use --prob 1.0 to ensure all are kept if logic is correct.

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "*",
            "--prob",
            "1.0",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Header + 3 distinct rows
    assert_eq!(lines.len(), 4);
}
