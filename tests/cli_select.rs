use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn select_fields_by_index_without_header() -> anyhow::Result<()> {
    let input = "a\tb\tc\n1\t2\t3\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("1,3")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "a\tc\n1\t3\n");

    Ok(())
}

#[test]
fn select_fields_by_name_with_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-H")
        .arg("-f")
        .arg("field3,field1")
        .arg("tests/data/select/input_header1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "field3\tfield1\n13567\t11567\n23567\t21567\n"
    );

    Ok(())
}

#[test]
fn select_fields_by_name_with_header_wildcard() -> anyhow::Result<()> {
    let input = "run\telapsed_time\tuser_time\tsystem_time\tmax_memory\n1\t57.5\t52.0\t5.5\t1420\n2\t52.0\t49.0\t3.0\t1270\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-H")
        .arg("-f")
        .arg("*_time")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "elapsed_time\tuser_time\tsystem_time\n57.5\t52.0\t5.5\n52.0\t49.0\t3.0\n"
    );

    Ok(())
}

#[test]
fn select_fields_by_name_with_header_name_range() -> anyhow::Result<()> {
    let input = "run\telapsed_time\tuser_time\tsystem_time\tmax_memory\n1\t57.5\t52.0\t5.5\t1420\n2\t52.0\t49.0\t3.0\t1270\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-H")
        .arg("-f")
        .arg("run-user_time")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "run\telapsed_time\tuser_time\n1\t57.5\t52.0\n2\t52.0\t49.0\n"
    );

    Ok(())
}

#[test]
fn select_exclude_field_by_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-e")
        .arg("2")
        .arg("tests/data/select/input_3x3.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "f1\tf3\n3x3-r1\t31\n3x3-r2\t32\n3x3-r3\t33\n"
    );

    Ok(())
}

#[test]
fn select_reorders_fields_on_file_input() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("2,1")
        .arg("tests/data/select/input_2fields.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "f2\tf1\ndef\tabc\n456\t123\nDEF\tABC\n");

    Ok(())
}

#[test]
fn select_field_from_input1_by_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("1")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "f1\n1\n\n3\n4\n5\n6\n7\n8\n");

    Ok(())
}

#[test]
fn select_field_range_from_input1() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("2-3")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "f2\tf3\nggg\tUUU\nf1-empty\tCCC\nßßß\tSSS\nsss\tf4-empty\nÀBC\t\n\t\n \t \n0.0\tZ\n"
    );

    Ok(())
}

#[test]
fn select_exclude_first_field_from_input1() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-e")
        .arg("1")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "f2\tf3\tf4\nggg\tUUU\t101\nf1-empty\tCCC\t5734\nßßß\tSSS\t 7\nsss\tf4-empty\nÀBC\t\t1367\n\t\tf23-empty\n \t \tf23-space\n0.0\tZ\t1931\n"
    );

    Ok(())
}

#[test]
fn select_with_alternate_delimiter_hat() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("1")
        .arg("--delimiter")
        .arg("^")
        .arg("tests/data/select/input_2plus_hat_delim.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "f1\nabc\n\n\n123\n\n");

    Ok(())
}

#[test]
fn select_with_alternate_delimiter_hat_second_field() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("2")
        .arg("--delimiter")
        .arg("^")
        .arg("tests/data/select/input_2plus_hat_delim.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "f2\ndef\n\n\n456\nabc\n");

    Ok(())
}

#[test]
fn select_from_empty_file_without_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("1")
        .arg("tests/data/select/input_emptyfile.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "");

    Ok(())
}

#[test]
fn select_from_empty_file_with_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-H")
        .arg("-f")
        .arg("1")
        .arg("tests/data/select/input_emptyfile.tsv")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "\n");

    Ok(())
}

#[test]
fn select_from_multiple_files_without_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("2,1")
        .arg("tests/data/select/input_3x2.tsv")
        .arg("tests/data/select/input_emptyfile.tsv")
        .arg("tests/data/select/input_3x1.tsv")
        .arg("tests/data/select/input_3x0.tsv")
        .arg("tests/data/select/input_3x3.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout,
        "f2\tf1\n2001\t3x2-r1\n2002\t3x2-r2\nf2\tf1\n201\t3x1-r1\nf2\tf1\nf2\tf1\n21\t3x3-r1\n22\t3x3-r2\n23\t3x3-r3\n"
    );

    Ok(())
}

#[test]
fn select_from_multiple_files_with_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-H")
        .arg("-f")
        .arg("2")
        .arg("tests/data/select/input_header1.tsv")
        .arg("tests/data/select/input_header2.tsv")
        .arg("tests/data/select/input_header3.tsv")
        .arg("tests/data/select/input_header4.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "field2\n12567\n22567\n12987\n12888\n22888\n");

    Ok(())
}

#[test]
fn select_requires_fields_or_exclude() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("one of --fields/-f or --exclude/-e is required"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn select_cannot_use_fields_and_exclude_together() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("1")
        .arg("-e")
        .arg("2")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("--fields/-f and --exclude/-e cannot be used together"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn select_error_zero_field_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("0")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn select_error_trailing_comma_in_field_list() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("1,")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("empty field list element"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn select_error_name_without_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-f")
        .arg("field1")
        .arg("tests/data/select/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("requires header"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn select_error_unknown_field_name_with_header_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-H")
        .arg("-f")
        .arg("no_such_field")
        .arg("tests/data/select/input_header1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("unknown field name `no_such_field`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn select_error_unknown_field_name_with_header_exclude() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("select")
        .arg("-H")
        .arg("-e")
        .arg("no_such_field")
        .arg("tests/data/select/input_header1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("unknown field name `no_such_field`"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}
