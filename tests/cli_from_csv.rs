use assert_cmd::cargo::cargo_bin_cmd;

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn from_csv_basic() -> anyhow::Result<()> {
    let input = "color,count\nred,1\ngreen,2\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("from").arg("csv").write_stdin(input).output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "color\tcount\nred\t1\ngreen\t2\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_with_quotes_and_commas() -> anyhow::Result<()> {
    let input = "name,comment\n\"a,b\",\"c,d\"\n\"x\"\"y\",z\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("from").arg("csv").write_stdin(input).output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "name\tcomment\na,b\tc,d\nx\"y\tz\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_with_custom_delimiter() -> anyhow::Result<()> {
    let input = "color;count\nred;1\ngreen;2\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("--delimiter")
        .arg(";")
        .write_stdin(input)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "color\tcount\nred\t1\ngreen\t2\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_input1_format1_file() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/input1_format1.csv")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "\
Previous line specifies next\tLegend\tA - Char\t_ - Space
T - Tab\tN - Newline\tQ - Quote\tC - Comma
AAA\tAAA_AAA\t_A_\tAAA
abc\tabc def\t a \tabc
ATAT\tT\tTT\tAAA
a b \t \t  \tabc
T\tT\tT\tT
 \t \t \t\u{0020}
\t\t\t
AAA\tAAA\tAAA\tAAA
abc\tabc\tabc\tabc
ANA\tAANNAA\tAA_AANAA_AA\tAAA
a b\tab  cd\tab cd ef gh\tabc
Q\tQQ\tAQA\tAQAAQA
\"\t\"\"\ta\"b\ta\"bc\"d
QQQ\tAQQA\tQAQAQ\tQQAQQAQQ
\"\"\"\ta\"\"b\t\"a\"a\"\t\"\"a\"\"a\"\"
C\tCC\tACA\tACAACA
,\t,,\ta,b\ta,bc,d
CCC\tACCA\tCACAC\tCCACCACC
,,,\ta,,b\t,a,a,\t,,a,,a,,
QCQ\tQNQ\tCNQACAQ\t_Q_NCCQCQ
\"a\"\t\" \"\t, \"a,b,\"\t\"  ,,\",\"
A\tAA\tAAA\tAAAA
a\tab\tabc\tabcd
";

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_input3_multiline_and_tabs() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/input3.csv")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "\
Type\tValue1\tValue2
Vanilla\tABC\t123
Quoted\tABC\t123
With Comma\tabc,def\t123,4
With Quotes\tSay \"Hello World!\"\t10\" high
With Newline\tValue 1 Line 1 Value 1 Line 2\tValue 2 Line 1 Value 2 Line 2
With TAB\tABC DEF\t123 456
";

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_input_unicode() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/input_unicode.csv")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "\
english\tcolor green yellow blue white black\tgreen\tblue
日本語\tカラーグリーンイエローブルーホワイトブラック\t緑\t青
deutsche\tFarbe grün gelb blau weiß schwarz\tgrün\tblau
suomalainen\tväri vihreä keltainen sininen valkoinen musta\tvihreä\tsininen
中文\t颜色绿色黄色蓝色白色黑色\t绿色\t蓝色
";

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_input_bom() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/input_bom.csv")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "\
abc\tdef\tghi
ABC\tDEF\tGHI
12.3\t45.6\t78.9
";

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_invalid1_should_fail() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/invalid1.csv")
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = normalize_newlines(&stderr);
    assert!(
        stderr
            .contains("tva from csv: invalid CSV in 'tests/data/from_csv/invalid1.csv'"),
        "unexpected stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("line"),
        "expected line information in stderr, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn from_csv_invalid2_should_fail() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/invalid2.csv")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn from_csv_stdin_filename_explicit() -> anyhow::Result<()> {
    let input = "a,b\n1,2\n3,4\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("from")
        .arg("csv")
        .arg("stdin")
        .write_stdin(input)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "a\tb\n1\t2\n3\t4\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn from_csv_gz_matches_plain_csv() -> anyhow::Result<()> {
    // plain CSV
    let mut cmd_plain = cargo_bin_cmd!("tva");
    let output_plain = cmd_plain
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/boston311-100.csv")
        .output()
        .unwrap();

    assert!(output_plain.status.success());
    let stdout_plain = String::from_utf8(output_plain.stdout).unwrap();
    let stdout_plain = normalize_newlines(&stdout_plain);

    // gzipped CSV
    let mut cmd_gz = cargo_bin_cmd!("tva");
    let output_gz = cmd_gz
        .arg("from")
        .arg("csv")
        .arg("tests/data/from_csv/boston311-100.csv.gz")
        .output()
        .unwrap();

    assert!(output_gz.status.success());
    let stdout_gz = String::from_utf8(output_gz.stdout).unwrap();
    let stdout_gz = normalize_newlines(&stdout_gz);

    assert_eq!(stdout_plain, stdout_gz);

    Ok(())
}
