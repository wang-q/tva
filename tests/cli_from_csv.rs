#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn from_csv_basic() {
    let input = "color,count\nred,1\ngreen,2\n";

    let (stdout, _) = TvaCmd::new().stdin(input).args(&["from", "csv"]).run();
    let stdout = normalize_newlines(&stdout);

    let expected = "color\tcount\nred\t1\ngreen\t2\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_csv_with_quotes_and_commas() {
    let input = "name,comment\n\"a,b\",\"c,d\"\n\"x\"\"y\",z\n";

    let (stdout, _) = TvaCmd::new().stdin(input).args(&["from", "csv"]).run();
    let stdout = normalize_newlines(&stdout);

    let expected = "name\tcomment\na,b\tc,d\nx\"y\tz\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_csv_with_custom_delimiter() {
    let input = "color;count\nred;1\ngreen;2\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["from", "csv", "--delimiter", ";"])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "color\tcount\nred\t1\ngreen\t2\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_csv_input1_format1_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/input1_format1.csv"])
        .run();
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
}

#[test]
fn from_csv_input3_multiline_and_tabs() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/input3.csv"])
        .run();
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
}

#[test]
fn from_csv_input_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/input_unicode.csv"])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "\
english\tcolor green yellow blue white black\tgreen\tblue
日本語\tカラーグリーンイエローブルーホワイトブラック\t緑\t青
deutsche\tFarbe grün gelb blau weiß schwarz\tgrün\tblau
suomalainen\tväri vihreä keltainen sininen valkoinen musta\tvihreä\tsininen
中文\t颜色绿色黄色蓝色白色黑色\t绿色\t蓝色
";

    assert_eq!(stdout, expected);
}

#[test]
fn from_csv_input_bom() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/input_bom.csv"])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "\
abc\tdef\tghi
ABC\tDEF\tGHI
12.3\t45.6\t78.9
";

    assert_eq!(stdout, expected);
}

#[test]
fn from_csv_invalid1_should_fail() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/invalid1.csv"])
        .run_fail();
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
}

#[test]
fn from_csv_invalid2_should_fail() {
    let (_stdout, _stderr) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/invalid2.csv"])
        .run();
}

#[test]
fn from_csv_stdin_filename_explicit() {
    let input = "a,b\n1,2\n3,4\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["from", "csv", "stdin"])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "a\tb\n1\t2\n3\t4\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_csv_gz_matches_plain_csv() {
    // plain CSV
    let (stdout_plain, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/boston311-100.csv"])
        .run();
    let stdout_plain = normalize_newlines(&stdout_plain);

    // gzipped CSV
    let (stdout_gz, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/boston311-100.csv.gz"])
        .run();
    let stdout_gz = normalize_newlines(&stdout_gz);

    assert_eq!(stdout_plain, stdout_gz);
}

#[test]
fn from_csv_invalid_delimiter_length() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--delimiter", "TAB"])
        .stdin("a,b\n1,2\n")
        .run_fail();

    assert!(stderr.contains("delimiter must be a single byte"));
}

#[test]
fn from_csv_empty_records() {
    let input = "a,b\n\n1,2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "csv"])
        .stdin(input)
        .run();

    assert!(stdout.contains("a\tb\n1\t2\n"));
}

#[test]
fn from_csv_stdin_error() {
    let input = "a,b\n1,2,3\n";
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("tva from csv: invalid CSV at line"));
}

#[test]
fn from_csv_file_error_no_line_info() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/invalid1.csv"])
        .run_fail();

    assert!(
        stderr.contains("tva from csv: invalid CSV in 'tests/data/from_csv/invalid1.csv'")
    );
}
