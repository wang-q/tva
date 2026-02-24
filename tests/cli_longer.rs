use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn longer_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input1.tsv")
        .arg("--cols")
        .arg("2-4")
        .output()?;

    let expected = "ID\tname\tvalue\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\n";

    // Normalize line endings for Windows
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_interleaved() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_interleaved.tsv")
        .arg("--cols")
        .arg("2,4")
        .output()?;

    let expected = "ID\tExtra\tname\tvalue\nA\tx\tM1\t1\nA\tx\tM2\t2\nB\ty\tM1\t3\nB\ty\tM2\t4\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_quotes() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_quotes.tsv")
        .arg("--cols")
        .arg("2-3")
        .output()?;

    // Note: tva currently treats quotes as part of the header name in TSV
    let expected = "ID\tname\tvalue\nA\t\"col 1\"\t1\nA\tcol 2\t2\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_mixed() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_mixed.tsv")
        .arg("--cols")
        .arg("2-3")
        .output()?;

    let expected = "ID\tname\tvalue\nA\tnum\t1\nA\ttext\tfoo\nB\tnum\t2\nB\ttext\tbar\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_cols_by_name_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_mixed.tsv")
        .arg("--cols")
        .arg("num-text")
        .output()?;

    let expected = "ID\tname\tvalue\nA\tnum\t1\nA\ttext\tfoo\nB\tnum\t2\nB\ttext\tbar\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_cols_by_wildcard() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input1.tsv")
        .arg("--cols")
        .arg("Q*")
        .output()?;

    let expected = "ID\tname\tvalue\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_dup_cols() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_dup_cols.tsv")
        .arg("--cols")
        .arg("2-3")
        .output()?;

    // When selecting by index (2-3), it should pick the 2nd and 3rd columns
    // regardless of their names being identical.
    // The "name" column in output will contain the column header names.
    // Since both are "val", we expect "val" in the name column for both.

    let expected = "ID\textra\tname\tvalue\nA\tx\tval\t1\nA\tx\tval\t2\nB\ty\tval\t3\nB\ty\tval\t4\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_output_order() -> anyhow::Result<()> {
    // Verifies that output is row-major:
    // For each input row, it outputs all melted columns in order.
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input1.tsv")
        .arg("--cols")
        .arg("2-3") // Only Q1 and Q2
        .output()?;

    let expected = "ID\tQ3\tname\tvalue\nA\t3\tQ1\t1\nA\t3\tQ2\t2\nB\t6\tQ1\t4\nB\t6\tQ2\t5\nC\t9\tQ1\t7\nC\t9\tQ2\t8\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_empty_input() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_empty.tsv")
        .arg("--cols")
        .arg("2-3")
        .output()?;

    let expected = "ID\tname\tvalue\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_invalid_col() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input1.tsv")
        .arg("--cols")
        .arg("99")
        .output()?;

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("Invalid column index"));

    Ok(())
}

#[test]
fn longer_keep_na() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_na.tsv")
        .arg("--cols")
        .arg("2-3")
        .output()?;

    let expected = "ID\tname\tvalue\nF\tQ1\t16\nF\tQ2\t\nG\tQ1\t\nG\tQ2\t17\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_multi_id() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_multi_id.tsv")
        .arg("--cols")
        .arg("3-4")
        .output()?;

    let expected = "ID\tCategory\tname\tvalue\nA\tX\tQ1\t1\nA\tX\tQ2\t2\nB\tY\tQ1\t3\nB\tY\tQ2\t4\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_custom_names() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input1.tsv")
        .arg("--cols")
        .arg("2-4")
        .arg("--names-to")
        .arg("Question")
        .arg("--values-to")
        .arg("Answer")
        .output()?;

    let expected = "ID\tQuestion\tAnswer\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_drop_na() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input_na.tsv")
        .arg("--cols")
        .arg("2-3")
        .arg("--values-drop-na")
        .output()?;

    let expected = "ID\tname\tvalue\nF\tQ1\t16\nG\tQ2\t17\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn longer_multiple_files() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("longer")
        .arg("tests/data/longer/input1.tsv")
        .arg("tests/data/longer/input2.tsv")
        .arg("--cols")
        .arg("2-4")
        .output()?;

    let expected = "ID\tname\tvalue\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\nD\tQ1\t10\nD\tQ2\t11\nD\tQ3\t12\nE\tQ1\t13\nE\tQ2\t14\nE\tQ3\t15\n";

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);

    Ok(())
}
