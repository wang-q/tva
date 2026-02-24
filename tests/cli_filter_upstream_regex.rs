    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_iregex_4_abc() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--iregex")
        .arg("4:abc")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_iregex_3_sz() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--iregex")
        .arg("3:ß")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_iregex_4_sz() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--iregex")
        .arg("4:ß")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_not_iregex_4_abc() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--not-iregex")
        .arg("4:abc")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_not_iregex_f4_abc() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("-H")
        .arg("--not-iregex")
        .arg("F4:abc")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_or_iregex_input4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--or")
        .arg("--iregex")
        .arg("7,3,2:^.*b.*d$")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "4\taadd\taabdd\t10\t30\t15\tabd\t25\t25\n",
        "8\tbd\t\t10\t20\t40\tbcd\t15\t25\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
        "11\tAADD\tAABDD\t10\t30\t15\tABD\t25\t25\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}
