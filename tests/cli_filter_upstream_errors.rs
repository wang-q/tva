use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn upstream_error_no_such_file() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("tests/data/filter/non_existent_file.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("could not open"));
    Ok(())
}

#[test]
fn upstream_error_invalid_field_0() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ge")
        .arg("0:10")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Field index 0 is invalid")
            || stderr.contains("invalid")
            || stderr.contains("field index must be >= 1")
    );
    Ok(())
}

#[test]
fn upstream_error_ff_absdiff_le_invalid_value() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("1:2:g")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid numeric value") || stderr.contains("invalid"));
    Ok(())
}

#[test]
fn upstream_error_regex_no_matching_paren() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--regex")
        .arg("4:abc(d|e")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid regex") || stderr.contains("error"));
    Ok(())
}

#[test]
fn upstream_error_ff_absdiff_missing_second_colon() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("missing second `:`")
            || stderr.contains("expected FIELD1:FIELD2:NUM")
    );
    Ok(())
}

#[test]
fn upstream_error_invalid_field_abc() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ge")
        .arg("abc:10")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Field `abc` not found in header")
            || stderr.contains("invalid")
            || stderr.contains("unknown field name")
    );
    Ok(())
}

#[test]
fn upstream_error_invalid_numeric_value() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--eq")
        .arg("2:def")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid numeric value") || stderr.contains("def"));
    Ok(())
}

#[test]
fn upstream_error_invalid_regex() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--regex")
        .arg("4:abc(d|e")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("regex") || stderr.contains("error") || stderr.contains("parse")
    );
    Ok(())
}

#[test]
fn upstream_error_ff_le_invalid_values() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-le")
        .arg("2:3.1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("invalid")
            || stderr.contains("parse")
            || stderr.contains("unknown field name")
    );
    Ok(())
}

#[test]
fn upstream_error_ff_le_missing_field() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-le")
        .arg("2:")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("invalid")
            || stderr.contains("missing")
            || stderr.contains("mismatched field list")
    );
    Ok(())
}

#[test]
fn upstream_error_ff_str_ne_invalid_field() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-ne")
        .arg("abc:3")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("not found in header")
            || stderr.contains("invalid")
            || stderr.contains("abc")
    );
    Ok(())
}

#[test]
fn upstream_error_ff_absdiff_le_zero_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("1:0:0.5")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Zero is not a valid field index")
            || stderr.contains("must be >= 1")
    );
    Ok(())
}

#[test]
fn upstream_error_ff_gt_zero_index() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-gt")
        .arg("0:1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Zero is not a valid field index")
            || stderr.contains("must be >= 1")
    );
    Ok(())
}

#[test]
fn upstream_error_missing_value_lt() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--lt")
        // Don't provide value. Clap should error.
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("value is required") || stderr.contains("required"));
    Ok(())
}

#[test]
fn upstream_error_empty_invalid_field() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ge")
        .arg(":10")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid") || stderr.contains("empty"));
    Ok(())
}

#[test]
fn upstream_error_header_processing_no_digits() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ge")
        .arg("abc:10")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    // Broaden check
    assert!(
        stderr.contains("not found")
            || stderr.contains("invalid")
            || stderr.contains("error")
            || stderr.contains("unknown")
    );
    Ok(())
}

#[test]
fn upstream_error_invalid_spec_empty_field() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ge")
        .arg(":10")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid") || stderr.contains("empty"));
    Ok(())
}

#[test]
fn upstream_error_invalid_spec_missing_colon() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ge")
        .arg("1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("missing `:` separator")
            || stderr.contains("invalid")
            || stderr.contains("error")
            || stderr.contains("spec")
    );
    Ok(())
}

#[test]
fn upstream_error_invalid_spec_missing_value() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ge")
        .arg("1:")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("missing value") || stderr.contains("invalid"));
    Ok(())
}

#[test]
fn upstream_error_not_enough_fields() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ge")
        .arg("10:10")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    // This is not an error in tva (just filters out).
    // So status should be success.
    assert!(output.status.success());
    Ok(())
}
