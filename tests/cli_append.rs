#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn append_basic_input3x2_input3x5() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_basic_input1x3_input1x4() {
    let expected = "field1
row 1
row 2
field1
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_basic_four_files() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
field1
row 1
row 2
field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
field1
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_basic_single_file() {
    let expected = "field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_two_files() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--header",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_two_single_column_files() {
    let expected = "field1
row 1
row 2
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_four_files() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
row 1
row 2
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_single_file() {
    let expected = "field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-H", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_two_files() {
    let expected = "input3x2\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input3x5\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--track-source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_two_single_column_files() {
    let expected = "input1x3\tfield1
input1x3\trow 1
input1x3\trow 2
input1x4\tfield1
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-t",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_four_files() {
    let expected = "input3x2\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input1x3\tfield1
input1x3\trow 1
input1x3\trow 2
input3x5\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
input1x4\tfield1
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-t",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_single_file() {
    let expected = "input3x5\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-t", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_two_files() {
    let expected = "file\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--header",
            "--track-source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_two_single_column_files() {
    let expected = "file\tfield1
input1x3\trow 1
input1x3\trow 2
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-t",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_four_files() {
    let expected = "file\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input1x3\trow 1
input1x3\trow 2
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-t",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_single_file() {
    let expected = "file\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-H", "-t", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_two_files() {
    let expected = "source\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--source-header",
            "source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_two_single_column_files() {
    let expected = "source\tfield1
input1x3\trow 1
input1x3\trow 2
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--source-header",
            "source",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_four_files() {
    let expected = "source\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input1x3\trow 1
input1x3\trow 2
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-s",
            "source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_single_file() {
    let expected = "source\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-s", "source", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--track-source", "tests/data/append/input3x2.tsv"])
        .run();

    assert!(stdout.contains("input3x2\tfield1\tfield2\tfield3"));
}

#[test]
fn append_source_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--source-header",
            "filename",
            "tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.starts_with("filename\tfield1\tfield2\tfield3\n"));
}

#[test]
fn append_file_mapping() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--file",
            "custom_label=tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.contains("custom_label\tfield1\tfield2\tfield3"));
}

#[test]
fn append_invalid_file_mapping() {
    let (_stdout, stderr) = TvaCmd::new()
        .args(&["append", "--file", "invalid_format"])
        .run_fail();

    assert!(
        stderr.contains("invalid --file value `invalid_format`; expected LABEL=FILE")
    );
}

#[test]
fn append_invalid_delimiter() {
    let (_stdout, stderr) = TvaCmd::new()
        .args(&["append", "--delimiter", "TooLong"])
        .run_fail();

    assert!(stderr.contains("delimiter must be a single byte"));
}

#[test]
fn append_stdin_default() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--track-source"])
        .stdin("field1\tfield2\nval1\tval2\n")
        .run();

    assert!(stdout.contains("stdin\tfield1\tfield2"));
}

#[test]
fn append_custom_delimiter() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--track-source",
            "--delimiter",
            ":",
            "tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.contains("input3x2:field1\tfield2\tfield3"));
}

#[test]
fn append_subdir_filename_label() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--track-source", "tests/data/append/input3x2.tsv"])
        .run();

    assert!(stdout.contains("input3x2\tfield1"));
}

#[test]
fn append_mixed_order_pos_flag() {
    // A -f B
    // input1x3: row 1\nrow 2
    // input1x4: next-empty\n\nlast-line
    // --file implies --track-source, so source column is added.
    let expected = "file\tfield1
input1x3\trow 1
input1x3\trow 2
L\tnext-empty
L\t
L\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "tests/data/append/input1x3.tsv",
            "--file",
            "L=tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_mixed_order_flag_pos() {
    // -f B A
    let expected = "file\tfield1
L\tnext-empty
L\t
L\tlast-line
input1x3\trow 1
input1x3\trow 2
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "--file",
            "L=tests/data/append/input1x4.tsv",
            "tests/data/append/input1x3.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_mixed_order_pos_flag_pos() {
    // A -f B C
    // input3x2 (3 lines) -> input1x4 (3 lines) -> input3x5 (4 lines)
    // -H to strip headers
    let expected = "file\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
L\tnext-empty
L\t
L\tlast-line
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "tests/data/append/input3x2.tsv",
            "--file",
            "L=tests/data/append/input1x4.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_unicode_header_and_source_labels() {
    let expected = "πηγή\tfield1
κόκκινος\trow 1
κόκκινος\trow 2
άσπρο\tnext-empty
άσπρο\t
άσπρο\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-t",
            "-s",
            "πηγή",
            "-f",
            "κόκκινος=tests/data/append/input1x3.tsv",
            "-f",
            "άσπρο=tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_empty_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "tests/data/append/empty-file.txt"])
        .run();
    assert!(stdout.is_empty());
}

#[test]
fn append_header_empty_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-H", "tests/data/append/empty-file.txt"])
        .run();
    assert!(stdout.is_empty());
}

#[test]
fn append_stdin_pipe() {
    let input = "field1\tfield2\tfield3\nabc\tdef\tghi\n";
    let expected = "field1\tfield2\tfield3\nabc\tdef\tghi\n";
    let (stdout, _) = TvaCmd::new().args(&["append"]).stdin(input).run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_stdin_dash_arg_middle() {
    let stdin_input = "field1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n";
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--", "tests/data/append/input3x2.tsv", "-"])
        .stdin(stdin_input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_stdin_dash_arg_middle_with_header() {
    let stdin_input = "field1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n";
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-H", "--", "tests/data/append/input3x2.tsv", "-"])
        .stdin(stdin_input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_stdin_explicit_file_mapping() {
    let stdin_input = "field1\tfield2\tfield3\nabc\tdef\tghi\n";
    // 3x5: jkl..., 123...
    // standard-input: abc...
    // 3x5: jkl... (again, but with label '3x5')
    let expected = "file\tfield1\tfield2\tfield3
standard-input\tabc\tdef\tghi
3x5\tjkl\tmno\tpqr
3x5\t123\t456\t789
3x5\txy1\txy2\txy3
3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-f",
            "standard-input=-",
            "-f",
            "3x5=tests/data/append/input3x5.tsv",
        ])
        .stdin(stdin_input)
        .run();
    assert_eq!(stdout, expected);
}
