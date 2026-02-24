use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn filter_numeric_gt_basic() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t20\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("value:10")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tvalue");
    assert_eq!(lines[1], "2\t15");
    assert_eq!(lines[2], "3\t20");

    Ok(())
}

#[test]
fn filter_str_eq_basic() -> anyhow::Result<()> {
    let input = "id\tcolor\n1\tred\n2\tblue\n3\tred\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--str-eq")
        .arg("color:red")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tcolor");
    assert_eq!(lines[1], "1\tred");
    assert_eq!(lines[2], "3\tred");

    Ok(())
}

#[test]
fn filter_regex_basic() -> anyhow::Result<()> {
    let input = "id\tname\n1\talice\n2\tbob\n3\talex\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--regex")
        .arg("name:^al")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "1\talice");
    assert_eq!(lines[2], "3\talex");

    Ok(())
}

#[test]
fn filter_not_regex_basic() -> anyhow::Result<()> {
    let input = "id\tname\n1\talice\n2\tbob\n3\talex\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--not-regex")
        .arg("name:^al")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "2\tbob");

    Ok(())
}

#[test]
fn filter_char_len_gt_basic() -> anyhow::Result<()> {
    let input = "id\tname\n1\ta\n2\tabcd\n3\txyz\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--char-len-gt")
        .arg("name:3")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "2\tabcd");

    Ok(())
}

#[test]
fn filter_byte_len_lt_basic() -> anyhow::Result<()> {
    let input = "id\tname\n1\ta\n2\tabcd\n3\txyz\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--byte-len-lt")
        .arg("name:4")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "1\ta");
    assert_eq!(lines[2], "3\txyz");

    Ok(())
}

#[test]
fn filter_istr_not_in_fld_basic() -> anyhow::Result<()> {
    let input = "id\tdesc\n1\tHelloWorld\n2\tfoo\n3\tHELLO\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--istr-not-in-fld")
        .arg("desc:hello")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "id\tdesc");
    assert_eq!(lines[1], "2\tfoo");

    Ok(())
}

#[test]
fn filter_count_and_invert() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t20\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("value:10")
        .arg("--invert")
        .arg("--count")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.trim().starts_with("1"));

    Ok(())
}

#[test]
fn filter_invalid_field_list_reports_error() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("0:10")
        .write_stdin(input);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("tva filter:"));

    Ok(())
}

#[test]
fn filter_is_numeric_basic() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n2\tfoo\n3\t10.5\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--is-numeric")
        .arg("value")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tvalue");
    assert_eq!(lines[1], "1\t5");
    assert_eq!(lines[2], "3\t10.5");

    Ok(())
}

#[test]
fn filter_ff_eq_basic() -> anyhow::Result<()> {
    let input = "id\ta\tb\n1\t5\t5\n2\t3\t4\n3\t7\t7\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-eq")
        .arg("a:b")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\ta\tb");
    assert_eq!(lines[1], "1\t5\t5");
    assert_eq!(lines[2], "3\t7\t7");

    Ok(())
}

#[test]
fn filter_ff_str_ne_basic() -> anyhow::Result<()> {
    let input = "id\tx\ty\n1\tfoo\tfoo\n2\tbar\tbaz\n3\tHELLO\thello\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-ne")
        .arg("x:y")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tx\ty");
    assert_eq!(lines[1], "2\tbar\tbaz");
    assert_eq!(lines[2], "3\tHELLO\thello");

    Ok(())
}

#[test]
fn filter_ff_absdiff_le_basic() -> anyhow::Result<()> {
    let input = "id\ta\tb\n1\t1.0\t1.2\n2\t5.0\t5.6\n3\t10.0\t10.1\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("a:b:0.2")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\ta\tb");
    assert_eq!(lines[1], "1\t1.0\t1.2"); // |1.0-1.2| = 0.2
    assert_eq!(lines[2], "3\t10.0\t10.1"); // |10.0-10.1| = 0.1

    Ok(())
}

#[test]
fn filter_ff_reldiff_gt_basic() -> anyhow::Result<()> {
    let input = "id\ta\tb\n1\t10\t12\n2\t100\t110\n3\t50\t55\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-gt")
        .arg("a:b:0.09")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "id\ta\tb");
    assert_eq!(lines[1], "1\t10\t12"); // |2|/10 = 0.2 > 0.09
    assert_eq!(lines[2], "2\t100\t110"); // |10|/100 = 0.1 > 0.09
    assert_eq!(lines[3], "3\t50\t55"); // |5|/50 = 0.1 > 0.09

    Ok(())
}

#[test]
fn filter_label_basic() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t8\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("value:10")
        .arg("--label")
        .arg("pass")
        .arg("--label-values")
        .arg("Y:N")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "id\tvalue\tpass");
    assert_eq!(lines[1], "1\t5\tN");
    assert_eq!(lines[2], "2\t15\tY");
    assert_eq!(lines[3], "3\t8\tN");

    Ok(())
}
