use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn upstream_field_list_ge_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ge")
        .arg("4-6:25")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
        "5\tad\t\t30\t35\t25\tbcdef\t40\t15\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_blank_2_3() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--blank")
        .arg("2,3")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_empty_2_3_7() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--empty")
        .arg("2,3,7")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_str_eq_4_6_0() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--str-eq")
        .arg("4-6:0")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_str_in_fld_2_3_7_ab() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--str-in-fld")
        .arg("2-3,7:ab")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected =
        concat!("line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",);
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_str_in_fld_2_3_ab() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--str-in-fld")
        .arg("2-3:ab")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_str_not_in_fld_2_3_ab() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--str-not-in-fld")
        .arg("2-3:ab")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
        "5\tad\t\t30\t35\t25\tbcdef\t40\t15\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "7\tbcf\tcc\t-20\t-50\t0\tabc\t0\t-5\n",
        "8\tbd\t\t10\t20\t40\tbcd\t15\t25\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
        "11\tAADD\tAABDD\t10\t30\t15\tABD\t25\t25\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_or_istr_eq_2_3_abc_upper() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--or")
        .arg("--istr-eq")
        .arg("2-3:abc")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_or_istr_eq_2_3_abc() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--or")
        .arg("--istr-eq")
        .arg("2-3:ABC")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_istr_in_fld_2_3_ab() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--istr-in-fld")
        .arg("2-3:ab")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_or_regex_2_3_7() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--or")
        .arg("--regex")
        .arg("2-3,7:^.*b.*d$")
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
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_not_regex_2_3_7() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--not-regex")
        .arg("2-3,7:^.*b.*d$")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
        "5\tad\t\t30\t35\t25\tbcdef\t40\t15\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "7\tbcf\tcc\t-20\t-50\t0\tabc\t0\t-5\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
        "11\tAADD\tAABDD\t10\t30\t15\tABD\t25\t25\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_or_iregex_7_3_2() -> anyhow::Result<()> {
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

#[test]
fn upstream_field_list_gt_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("4-6:25")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_eq_reverse_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--eq")
        .arg("6-4,8:0")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_ne_ranges() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ne")
        .arg("4-6,8-9:0")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
        "4\taadd\taabdd\t10\t30\t15\tabd\t25\t25\n",
        "5\tad\t\t30\t35\t25\tbcdef\t40\t15\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "8\tbd\t\t10\t20\t40\tbcd\t15\t25\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
        "11\tAADD\tAABDD\t10\t30\t15\tABD\t25\t25\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_or_eq_ranges() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--or")
        .arg("--eq")
        .arg("4-6,8-9:0")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "7\tbcf\tcc\t-20\t-50\t0\tabc\t0\t-5\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_field_list_lt_list() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--lt")
        .arg("4,5:0")
        .arg("tests/data/filter/input4.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "7\tbcf\tcc\t-20\t-50\t0\tabc\t0\t-5\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}
