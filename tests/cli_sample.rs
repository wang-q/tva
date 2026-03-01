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
    // But critically, a_count must be 0 or 3. b_count must be 0 or 2.
    assert!(a_count == 0 || a_count == 3, "a_count was {}", a_count);
    assert!(b_count == 0 || b_count == 2, "b_count was {}", b_count);
}

#[test]
fn sample_weighted_shuffle() {
    // Weighted shuffle (no --num).
    // High weight items should appear earlier on average.
    // But for a deterministic test, we check that all items are present.
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
        // Wait, tsv-sample distinct sampling assigns random value per KEY.
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
    // B	1 should likely have different
    // (technically possible to be same collision, but unlikely with double)
    // But with static seed 42...
    // Let's just check they are valid numbers.
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
    // Row 1 and 2 share key (A,1) if we use k=1,2.
    // Wait, row 2 is A,1,Y. Row 1 is A,1,X.
    // If k=1,2, keys are "A", "1".
    // So row 1 and 2 are SAME key.

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
