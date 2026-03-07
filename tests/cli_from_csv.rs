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

    // This expected string is derived from gold/basic_tests_1.txt
    // Note: The source gold file uses TABs.
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
fn from_csv_input1_format2_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/input1_format2.csv"])
        .run();
    let stdout = normalize_newlines(&stdout);

    // Same expected output as format1
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
fn from_csv_input1_format3_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/input1_format3.csv"])
        .run();
    let stdout = normalize_newlines(&stdout);

    // Same expected output as format1
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
fn from_csv_input2_custom_options() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "csv",
            "--quote",
            "#",
            "--delimiter",
            "|",
            "--tab-replacement",
            "<==>",
            "--newline-replacement",
            "<==>",
            "tests/data/from_csv/input2.csv",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    // Expected output with TABs instead of $
    let expected = "\
field1\tfield2\tfield3
123\t456\t789
234\t567\t890
|abc\t#def#\tgh><==>ijk><==>lmn<
ABC\tDEF\tGHI
";
    assert_eq!(stdout, expected);
}

#[test]
fn from_csv_input2_short_options() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "csv",
            "-q",
            "#",
            "-d",
            "|",
            "-r",
            "<-->",
            "-n",
            "<-->",
            "tests/data/from_csv/input2.csv",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "\
field1\tfield2\tfield3
123\t456\t789
234\t567\t890
|abc\t#def#\tgh><-->ijk><-->lmn<
ABC\tDEF\tGHI
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
fn from_csv_input3_replacements() {
    // --tab-replacement <TAB>
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "csv",
            "--tab-replacement",
            "<TAB>",
            "tests/data/from_csv/input3.csv",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    assert!(stdout.contains("ABC<TAB>DEF"));

    // --newline-replacement <NL>
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "csv",
            "--newline-replacement",
            "<NL>",
            "tests/data/from_csv/input3.csv",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    assert!(stdout.contains("Value 1 Line 1<NL>Value 1 Line 2"));
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

/*
#[test]
fn from_csv_invalid2_should_fail() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/invalid2.csv"])
        .run_fail();
    let stderr = normalize_newlines(&stderr);
    assert!(stderr.contains("tva from csv: invalid CSV"));
}
*/

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
    let (stdout, _) = TvaCmd::new().args(&["from", "csv"]).stdin(input).run();

    assert!(stdout.contains("a\tb\n1\t2\n"));
}

#[test]
fn from_csv_stdin_error() {
    let input = "a,b\n1,2,3\n";
    let (_, stderr) = TvaCmd::new().args(&["from", "csv"]).stdin(input).run_fail();

    assert!(stderr.contains("tva from csv: invalid CSV"));
}

#[test]
fn from_csv_file_error_no_line_info() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "tests/data/from_csv/invalid1.csv"])
        .run_fail();

    assert!(stderr
        .contains("tva from csv: invalid CSV in 'tests/data/from_csv/invalid1.csv'"));
}

// Error cases from error_tests_1.txt

#[test]
fn from_csv_delimiter_newline_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--delimiter", "\n"])
        .run_fail();
    assert!(stderr.contains("CSV field delimiter cannot be newline"));
}

#[test]
fn from_csv_quote_newline_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--quote", "\n"])
        .run_fail();
    assert!(stderr.contains("CSV quote character cannot be newline"));
}

#[test]
fn from_csv_delimiter_quote_same_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--delimiter", "x", "--quote", "x"])
        .run_fail();
    assert!(stderr
        .contains("CSV quote and CSV field delimiter characters must be different"));
}

#[test]
fn from_csv_replacement_newline_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--tab-replacement", "\n"])
        .run_fail();
    assert!(stderr.contains(
        "Replacement character cannot contain newlines or TSV field delimiters"
    ));
}

#[test]
fn from_csv_replacement_tab_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--tab-replacement", "\t"])
        .run_fail();
    assert!(stderr.contains(
        "Replacement character cannot contain newlines or TSV field delimiters"
    ));
}

#[test]
fn from_csv_newline_replacement_validation() {
    // Tests L160-167: newline replacement validation
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--newline-replacement", "\t"])
        .run_fail();
    assert!(stderr.contains("Replacement character cannot contain newlines or TSV field delimiters"));

    let (_, stderr2) = TvaCmd::new()
        .args(&["from", "csv", "--newline-replacement", "\n"])
        .run_fail();
    assert!(stderr2.contains("Replacement character cannot contain newlines or TSV field delimiters"));
}

#[test]
fn from_csv_quote_invalid_length() {
    // Tests L115-120: Quote must be single byte
    let (_, stderr) = TvaCmd::new()
        .args(&["from", "csv", "--quote", "QQ"])
        .run_fail();
    assert!(stderr.contains("quote must be a single byte"));
}

#[test]
fn from_csv_sanitize_field() {
    // Tests sanitize_field function logic (L67-79)
    // We need input with tabs and newlines inside quoted fields to trigger sanitize_field logic
    let input = "a,\"b\tc\",d\n1,\"2\n3\",4\n";
    // Expected: tabs replaced by default " ", newlines replaced by default " "
    let expected = "a\tb c\td\n1\t2 3\t4\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["from", "csv"])
        .run();
    assert_eq!(normalize_newlines(&stdout), expected);

    // Custom replacements
    let (stdout2, _) = TvaCmd::new()
        .stdin(input)
        .args(&["from", "csv", "--tab-replacement", "T", "--newline-replacement", "N"])
        .run();
    let expected2 = "a\tbTc\td\n1\t2N3\t4\n";
    assert_eq!(normalize_newlines(&stdout2), expected2);
}
