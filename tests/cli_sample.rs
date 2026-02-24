use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[test]
fn sample_shuffle_basic() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);

    let mut lines: Vec<&str> = out_str.lines().collect();
    lines.sort();
    assert_eq!(lines, vec!["a", "b", "c", "d"]);

    Ok(())
}

#[test]
fn sample_num_limited() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = out_str.lines().collect();

    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_prob_keeps_subset() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--prob")
        .arg("0.5")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = out_str.lines().collect();

    assert!(lines.len() <= 4);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_header_preserved() -> anyhow::Result<()> {
    let input = "h1\n1\n2\n3\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--num")
        .arg("2")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let mut lines = out_str.lines();

    let header = lines.next().unwrap();
    assert_eq!(header, "h1");

    let data: Vec<&str> = lines.collect();
    assert_eq!(data.len(), 2);
    for line in &data {
        assert!(["1", "2", "3"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_invalid_prob_rejected() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample").arg("--prob").arg("0.0");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid --prob/-p value"));

    Ok(())
}

#[test]
fn sample_static_seed_produces_reproducible_output() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";

    let mut cmd1 = cargo_bin_cmd!("tva");
    let out1 = cmd1
        .arg("sample")
        .arg("--num")
        .arg("3")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();
    let s1 = String::from_utf8(out1.stdout).unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let out2 = cmd2
        .arg("sample")
        .arg("--num")
        .arg("3")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();
    let s2 = String::from_utf8(out2.stdout).unwrap();

    assert_eq!(s1, s2);

    Ok(())
}

#[test]
fn sample_replace_requires_num() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--replace")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("requires --num/-n greater than 0"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn sample_replace_conflicts_with_prob() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--replace")
        .arg("--num")
        .arg("2")
        .arg("--prob")
        .arg("0.5")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_replace_basic() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("10")
        .arg("--replace")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_inorder_requires_num() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--inorder")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_inorder_conflicts_with_prob() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .arg("--prob")
        .arg("0.5")
        .arg("--inorder")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_inorder_conflicts_with_replace() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .arg("--replace")
        .arg("--inorder")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_inorder_basic() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .arg("--inorder")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
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

    Ok(())
}

#[test]
fn sample_weight_field_basic() -> anyhow::Result<()> {
    let input = "x\t1\nx\t10\nx\t100\nx\t1000\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("1")
        .arg("--weight-field")
        .arg("2")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "x\t1000");

    Ok(())
}

#[test]
fn sample_weight_field_header_by_name() -> anyhow::Result<()> {
    let input = "name\tw\nx\t1\ny\t10\nz\t100\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--num")
        .arg("1")
        .arg("--weight-field")
        .arg("w")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "name\tw");

    Ok(())
}

#[test]
fn sample_weight_field_conflicts_with_prob_and_replace() -> anyhow::Result<()> {
    let input = "x\t1\nx\t10\n";

    let mut cmd1 = cargo_bin_cmd!("tva");
    let out1 = cmd1
        .arg("sample")
        .arg("--num")
        .arg("1")
        .arg("--weight-field")
        .arg("2")
        .arg("--prob")
        .arg("0.5")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(!out1.status.success());

    let mut cmd2 = cargo_bin_cmd!("tva");
    let out2 = cmd2
        .arg("sample")
        .arg("--num")
        .arg("1")
        .arg("--weight-field")
        .arg("2")
        .arg("--replace")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(!out2.status.success());

    Ok(())
}

#[test]
fn sample_weight_field_invalid_field_list_reports_error() -> anyhow::Result<()> {
    let input = "x\t1\nx\t10\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("1")
        .arg("--weight-field")
        .arg("0")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tva sample:"));

    Ok(())
}

#[test]
fn sample_key_fields_requires_prob() -> anyhow::Result<()> {
    let input = "k\tv\na\t1\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--key-fields")
        .arg("k")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_key_fields_distinct_per_key() -> anyhow::Result<()> {
    let input = "k\tv\na\t1\na\t2\nb\t3\nb\t4\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--prob")
        .arg("0.5")
        .arg("--key-fields")
        .arg("k")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
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

    Ok(())
}

#[test]
fn sample_gen_random_inorder_basic() -> anyhow::Result<()> {
    let input = "k\tv\na\t1\nb\t2\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--gen-random-inorder")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
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

    Ok(())
}

#[test]
fn sample_gen_random_inorder_conflicts_with_sampling() -> anyhow::Result<()> {
    let input = "a\nb\nc\n";

    let mut cmd1 = cargo_bin_cmd!("tva");
    let out1 = cmd1
        .arg("sample")
        .arg("--gen-random-inorder")
        .arg("--num")
        .arg("2")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(!out1.status.success());

    let mut cmd2 = cargo_bin_cmd!("tva");
    let out2 = cmd2
        .arg("sample")
        .arg("--gen-random-inorder")
        .arg("--prob")
        .arg("0.5")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(!out2.status.success());

    Ok(())
}

#[test]
fn sample_print_random_basic() -> anyhow::Result<()> {
    let input = "a\nb\nc\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--print-random")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    for line in &lines {
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols[0].parse::<f64>().is_ok());
    }

    Ok(())
}

#[test]
fn sample_print_random_not_allowed_with_replace() -> anyhow::Result<()> {
    let input = "a\nb\nc\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("5")
        .arg("--replace")
        .arg("--print-random")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_compat_num_superset() -> anyhow::Result<()> {
    let mut input = String::new();
    for i in 0..20 {
        input.push_str(&format!("{}\n", i));
    }

    let mut cmd_small = cargo_bin_cmd!("tva");
    let out_small = cmd_small
        .arg("sample")
        .arg("--compatibility-mode")
        .arg("--static-seed")
        .arg("--num")
        .arg("5")
        .write_stdin(input.clone())
        .output()
        .unwrap();
    assert!(out_small.status.success());
    let lines_small: HashSet<String> = String::from_utf8_lossy(&out_small.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    let mut cmd_large = cargo_bin_cmd!("tva");
    let out_large = cmd_large
        .arg("sample")
        .arg("--compatibility-mode")
        .arg("--static-seed")
        .arg("--num")
        .arg("10")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(out_large.status.success());
    let lines_large: HashSet<String> = String::from_utf8_lossy(&out_large.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    assert!(lines_small.is_subset(&lines_large));

    Ok(())
}

#[test]
fn sample_compat_multi_file_from_tsv_sample_inputs() -> anyhow::Result<()> {
    let base = PathBuf::from("tests/data/sample");
    let input1 = base.join("input3x10.tsv");
    let input2 = base.join("input3x25.tsv");

    let header_input = fs::read_to_string(&input1)?;
    let mut header_lines = header_input.lines();
    let expected_header = header_lines.next().unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--static-seed")
        .arg("--compatibility-mode")
        .arg(&input1)
        .arg(&input2)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut out_lines = stdout.lines();
    let header = out_lines.next().unwrap();
    assert_eq!(header, expected_header);

    fn count_data_rows(path: &PathBuf) -> anyhow::Result<usize> {
        let contents = fs::read_to_string(path)?;
        let mut it = contents.lines();
        let _ = it.next();
        Ok(it.count())
    }

    let expected_rows = count_data_rows(&input1)? + count_data_rows(&input2)?;
    let out_data: Vec<&str> = out_lines.collect();
    assert_eq!(out_data.len(), expected_rows);

    Ok(())
}

#[test]
fn sample_compat_stdin_and_files_from_tsv_sample_inputs() -> anyhow::Result<()> {
    let base = PathBuf::from("tests/data/sample");
    let stdin_path = base.join("input3x10.tsv");
    let file1 = base.join("input3x3.tsv");
    let file2 = base.join("input3x4.tsv");

    let stdin_data = fs::read_to_string(&stdin_path)?;

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--static-seed")
        .arg("--compatibility-mode")
        .arg("--")
        .arg("-")
        .arg(&file1)
        .arg(&file2)
        .write_stdin(stdin_data)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut out_lines = stdout.lines();
    let header = out_lines.next().unwrap();

    let stdin_header = fs::read_to_string(&stdin_path)?;
    let mut stdin_lines = stdin_header.lines();
    let expected_header = stdin_lines.next().unwrap();
    assert_eq!(header, expected_header);

    fn count_rows_with_header(path: &PathBuf, has_header: bool) -> anyhow::Result<usize> {
        let contents = fs::read_to_string(path)?;
        let mut it = contents.lines();
        if has_header {
            let _ = it.next();
        }
        Ok(it.count())
    }

    let expected_rows = count_rows_with_header(&stdin_path, true)?
        + count_rows_with_header(&file1, true)?
        + count_rows_with_header(&file2, true)?;

    let out_data: Vec<&str> = out_lines.collect();
    assert_eq!(out_data.len(), expected_rows);

    Ok(())
}

#[test]
fn sample_windows_newlines_from_tsv_sample_inputs() -> anyhow::Result<()> {
    let base = PathBuf::from("tests/data/sample");
    let unix_path = base.join("input3x25.tsv");
    let dos_path = base.join("input3x25.dos_tsv");

    let unix_contents = fs::read_to_string(&unix_path)?;
    let mut unix_lines = unix_contents.lines();
    let unix_header = unix_lines.next().unwrap();
    let unix_data_count = unix_lines.count();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg(&dos_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut out_lines = stdout.lines();
    let header = out_lines.next().unwrap();
    assert_eq!(header, unix_header);

    let out_data: Vec<&str> = out_lines.collect();
    assert_eq!(out_data.len(), unix_data_count);

    Ok(())
}
