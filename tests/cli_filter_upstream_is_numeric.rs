use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn upstream_is_numeric() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--is-numeric")
        .arg("2")
        .arg("tests/data/filter/input_numeric_tests.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\n",
        "1\tnan\n",
        "2\tNaN\n",
        "3\tNAN\n",
        "4\tinf\n",
        "5\t-inf\n",
        "6\tINF\n",
        "9\t23\n",
        "10\t-33.5\n",
        "11\t42.5\n",
        "12\t+45\n",
        "13\t.19\n",
        "14\t-.20\n",
        "15\t9e+02\n",
        "16\t8E-17\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_is_finite() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--is-finite")
        .arg("2")
        .arg("tests/data/filter/input_numeric_tests.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\n",
        "9\t23\n",
        "10\t-33.5\n",
        "11\t42.5\n",
        "12\t+45\n",
        "13\t.19\n",
        "14\t-.20\n",
        "15\t9e+02\n",
        "16\t8E-17\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_is_nan() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--is-nan")
        .arg("2")
        .arg("tests/data/filter/input_numeric_tests.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\n",
        "1\tnan\n",
        "2\tNaN\n",
        "3\tNAN\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_is_infinity() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--is-infinity")
        .arg("2")
        .arg("tests/data/filter/input_numeric_tests.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\n",
        "4\tinf\n",
        "5\t-inf\n",
        "6\tINF\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_is_numeric_combined() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--is-numeric")
        .arg("2")
        .arg("--gt")
        .arg("2:10")
        .arg("tests/data/filter/input_numeric_tests.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\n",
        "4\tinf\n",
        "6\tINF\n",
        "9\t23\n",
        "11\t42.5\n",
        "12\t+45\n",
        "15\t9e+02\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_is_finite_combined() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--is-finite")
        .arg("2")
        .arg("--gt")
        .arg("2:10")
        .arg("tests/data/filter/input_numeric_tests.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\n",
        "9\t23\n",
        "11\t42.5\n",
        "12\t+45\n",
        "15\t9e+02\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}
