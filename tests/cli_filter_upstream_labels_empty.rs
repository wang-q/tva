use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn upstream_label_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--label")
        .arg("Pass")
        .arg("--label-values")
        .arg("Y:N")
        .arg("--header")
        .arg("--eq")
        .arg("2:1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\tPass\n",
        "1\t1.0\ta\tA\tY\n",
        "2\t2.\tb\tB\tN\n",
        "10\t10.1\tabc\tABC\tN\n",
        "100\t100\tabc\tAbC\tN\n",
        "0\t0.0\tz\tAzB\tN\n",
        "-1\t-0.1\tabc def\tabc def\tN\n",
        "-2\t-2.0\tß\tss\tN\n",
        "0.\t100.\tàbc\tÀBC\tN\n",
        "0.0\t100.0\tàßc\tÀssC\tN\n",
        "-0.0\t-100.0\tàßc\tÀSSC\tN\n",
        "100\t100\t\tAbC\tN\n",
        "100\t100\tabc\t\tN\n",
        "100\t101\t\t\tN\n",
        "100\t102\tabc\tAbC\tN\n",
        "100\t103\tabc\tAbC\tN\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_label_values_custom() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--label")
        .arg("Pass")
        .arg("--label-values")
        .arg("Yes:No")
        .arg("--header")
        .arg("--eq")
        .arg("2:1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\tPass\n",
        "1\t1.0\ta\tA\tYes\n",
        "2\t2.\tb\tB\tNo\n",
        "10\t10.1\tabc\tABC\tNo\n",
        "100\t100\tabc\tAbC\tNo\n",
        "0\t0.0\tz\tAzB\tNo\n",
        "-1\t-0.1\tabc def\tabc def\tNo\n",
        "-2\t-2.0\tß\tss\tNo\n",
        "0.\t100.\tàbc\tÀBC\tNo\n",
        "0.0\t100.0\tàßc\tÀssC\tNo\n",
        "-0.0\t-100.0\tàßc\tÀSSC\tNo\n",
        "100\t100\t\tAbC\tNo\n",
        "100\t100\tabc\t\tNo\n",
        "100\t101\t\t\tNo\n",
        "100\t102\tabc\tAbC\tNo\n",
        "100\t103\tabc\tAbC\tNo\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_label_no_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--label")
        .arg("Pass")
        .arg("--label-values")
        .arg("Y:N")
        .arg("--eq")
        .arg("2:1")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "1\t1.0\ta\tA\tY\n",
        "2\t2.\tb\tB\tN\n",
        "10\t10.1\tabc\tABC\tN\n",
        "100\t100\tabc\tAbC\tN\n",
        "0\t0.0\tz\tAzB\tN\n",
        "-1\t-0.1\tabc def\tabc def\tN\n",
        "-2\t-2.0\tß\tss\tN\n",
        "0.\t100.\tàbc\tÀBC\tN\n",
        "0.0\t100.0\tàßc\tÀssC\tN\n",
        "-0.0\t-100.0\tàßc\tÀSSC\tN\n",
        "100\t100\t\tAbC\tN\n",
        "100\t100\tabc\t\tN\n",
        "100\t101\t\t\tN\n",
        "100\t102\tabc\tAbC\tN\n",
        "100\t103\tabc\tAbC\tN\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_empty_with_other_filter() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--eq")
        .arg("1:100")
        .arg("--empty")
        .arg("3")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "100\t100\t\tAbC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_not_blank_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--not-blank")
        .arg("3")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // input2.tsv has 14 lines (header + 13 data)
    // Blank 3rd field:
    // 1000	1000.0		3 empty
    // 1000	1000.0	 	3 1-space
    // 1000	1000.001	  	3 2-spaces
    // 3 lines blank. 10 lines remain.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 11); // Header + 10 lines
    assert_eq!(lines[0], "F1\tF2\tF3\tF4");
    Ok(())
}

#[test]
fn upstream_empty_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--empty")
        .arg("3")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "100\t100\t\tAbC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_not_empty_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--not-empty")
        .arg("3")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 14); // Header + 13 lines
    assert_eq!(lines[0], "F1\tF2\tF3\tF4");
    Ok(())
}

#[test]
fn upstream_blank_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--blank")
        .arg("3")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1000\t1000.0\t\t3 empty\n",
        "1000\t1000.0\t \t3 1-space\n",
        "1000\t1000.001\t  \t3 2-spaces\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}
