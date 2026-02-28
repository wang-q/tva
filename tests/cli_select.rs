#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn select_fields_by_index_without_header() {
    let input = "a\tb\tc\n1\t2\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "1,3"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "a\tc\n1\t3\n");
}

#[test]
fn select_fields_by_name_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-f",
            "field3,field1",
            "tests/data/select/input_header1.tsv",
        ])
        .run();

    assert_eq!(stdout, "field3\tfield1\n13567\t11567\n23567\t21567\n");
}

#[test]
fn select_fields_by_name_with_header_wildcard() {
    let input = "run\telapsed_time\tuser_time\tsystem_time\tmax_memory\n1\t57.5\t52.0\t5.5\t1420\n2\t52.0\t49.0\t3.0\t1270\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-H", "-f", "*_time"])
        .stdin(input)
        .run();

    assert_eq!(
        stdout,
        "elapsed_time\tuser_time\tsystem_time\n57.5\t52.0\t5.5\n52.0\t49.0\t3.0\n"
    );
}

#[test]
fn select_fields_by_name_with_header_name_range() {
    let input = "run\telapsed_time\tuser_time\tsystem_time\tmax_memory\n1\t57.5\t52.0\t5.5\t1420\n2\t52.0\t49.0\t3.0\t1270\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-H", "-f", "run-user_time"])
        .stdin(input)
        .run();

    assert_eq!(
        stdout,
        "run\telapsed_time\tuser_time\n1\t57.5\t52.0\n2\t52.0\t49.0\n"
    );
}

#[test]
fn select_fields_by_name_with_header_special_char_escapes() {
    let input = "test id\trun:id\ttime-stamp\t001\t100\nv1\tv2\tv3\tv4\tv5\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-f",
            r"test\ id,run\:id,time\-stamp,\001,\100",
        ])
        .stdin(input)
        .run();

    assert_eq!(
        stdout,
        "test id\trun:id\ttime-stamp\t001\t100\nv1\tv2\tv3\tv4\tv5\n"
    );
}

#[test]
fn select_handles_crlf_input_from_stdin() {
    let input = "f1\tf2\n1\t2\r\n3\t4\r\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "1,2", "-"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "f1\tf2\n1\t2\n3\t4\n");
}

#[test]
fn select_exclude_field_by_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-e", "2", "tests/data/select/input_3x3.tsv"])
        .run();

    assert_eq!(stdout, "f1\tf3\n3x3-r1\t31\n3x3-r2\t32\n3x3-r3\t33\n");
}

#[test]
fn select_reorders_fields_on_file_input() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "2,1", "tests/data/select/input_2fields.tsv"])
        .run();

    assert_eq!(stdout, "f2\tf1\ndef\tabc\n456\t123\nDEF\tABC\n");
}

#[test]
fn select_field_from_input1_by_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "1", "tests/data/select/input1.tsv"])
        .run();

    assert_eq!(stdout, "f1\n1\n\n3\n4\n5\n6\n7\n8\n");
}

#[test]
fn select_field_range_from_input1() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "2-3", "tests/data/select/input1.tsv"])
        .run();

    assert_eq!(
        stdout,
        "f2\tf3\nggg\tUUU\nf1-empty\tCCC\nßßß\tSSS\nsss\tf4-empty\nÀBC\t\n\t\n \t \n0.0\tZ\n"
    );
}

#[test]
fn select_exclude_first_field_from_input1() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-e", "1", "tests/data/select/input1.tsv"])
        .run();

    assert_eq!(
        stdout,
        "f2\tf3\tf4\nggg\tUUU\t101\nf1-empty\tCCC\t5734\nßßß\tSSS\t 7\nsss\tf4-empty\nÀBC\t\t1367\n\t\tf23-empty\n \t \tf23-space\n0.0\tZ\t1931\n"
    );
}

#[test]
fn select_exclude_large_index_is_noop() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-e", "1048576", "tests/data/select/input1.tsv"])
        .run();

    assert_eq!(
        stdout,
        "f1\tf2\tf3\tf4\n1\tggg\tUUU\t101\n\tf1-empty\tCCC\t5734\n3\tßßß\tSSS\t 7\n4\tsss\tf4-empty\n5\tÀBC\t\t1367\n6\t\t\tf23-empty\n7\t \t \tf23-space\n8\t0.0\tZ\t1931\n"
    );
}

#[test]
fn select_exclude_large_range_is_noop() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-e", "5-1048576", "tests/data/select/input1.tsv"])
        .run();

    assert_eq!(
        stdout,
        "f1\tf2\tf3\tf4\n1\tggg\tUUU\t101\n\tf1-empty\tCCC\t5734\n3\tßßß\tSSS\t 7\n4\tsss\tf4-empty\n5\tÀBC\t\t1367\n6\t\t\tf23-empty\n7\t \t \tf23-space\n8\t0.0\tZ\t1931\n"
    );
}

#[test]
fn select_with_alternate_delimiter_hat() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-f",
            "1",
            "--delimiter",
            "^",
            "tests/data/select/input_2plus_hat_delim.tsv",
        ])
        .run();

    assert_eq!(stdout, "f1\nabc\n\n\n123\n\n");
}

#[test]
fn select_with_alternate_delimiter_hat_second_field() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-f",
            "2",
            "--delimiter",
            "^",
            "tests/data/select/input_2plus_hat_delim.tsv",
        ])
        .run();

    assert_eq!(stdout, "f2\ndef\n\n\n456\nabc\n");
}

#[test]
fn select_from_empty_file_without_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "1", "tests/data/select/input_emptyfile.tsv"])
        .run();

    assert_eq!(stdout, "");
}

#[test]
fn select_from_empty_file_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-f",
            "1",
            "tests/data/select/input_emptyfile.tsv",
        ])
        .run();

    assert_eq!(stdout, "\n");
}

#[test]
fn select_from_multiple_files_without_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-f",
            "2,1",
            "tests/data/select/input_3x2.tsv",
            "tests/data/select/input_emptyfile.tsv",
            "tests/data/select/input_3x1.tsv",
            "tests/data/select/input_3x0.tsv",
            "tests/data/select/input_3x3.tsv",
        ])
        .run();

    assert_eq!(
        stdout,
        "f2\tf1\n2001\t3x2-r1\n2002\t3x2-r2\nf2\tf1\n201\t3x1-r1\nf2\tf1\nf2\tf1\n21\t3x3-r1\n22\t3x3-r2\n23\t3x3-r3\n"
    );
}

#[test]
fn select_from_multiple_files_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-f",
            "2",
            "tests/data/select/input_header1.tsv",
            "tests/data/select/input_header2.tsv",
            "tests/data/select/input_header3.tsv",
            "tests/data/select/input_header4.tsv",
        ])
        .run();

    assert_eq!(stdout, "field2\n12567\n22567\n12987\n12888\n22888\n");
}

#[test]
fn select_requires_fields_or_exclude() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "tests/data/select/input1.tsv"])
        .run_fail();

    assert!(stderr.contains("one of --fields/-f or --exclude/-e is required"));
}

#[test]
fn select_cannot_use_fields_and_exclude_together() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "select",
            "-f",
            "1",
            "-e",
            "2",
            "tests/data/select/input1.tsv",
        ])
        .run_fail();

    assert!(stderr.contains("--fields/-f and --exclude/-e cannot be used together"));
}

#[test]
fn select_error_zero_field_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "-f", "0", "tests/data/select/input1.tsv"])
        .run_fail();

    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn select_error_trailing_comma_in_field_list() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "-f", "1,", "tests/data/select/input1.tsv"])
        .run_fail();

    assert!(stderr.contains("empty field list element"));
}

#[test]
fn select_error_name_without_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "-f", "field1", "tests/data/select/input1.tsv"])
        .run_fail();

    assert!(stderr.contains("requires header"));
}

#[test]
fn select_error_unknown_field_name_with_header_fields() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-f",
            "no_such_field",
            "tests/data/select/input_header1.tsv",
        ])
        .run_fail();

    assert!(stderr.contains("unknown field name `no_such_field`"));
}

#[test]
fn select_error_unknown_field_name_with_header_exclude() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-e",
            "no_such_field",
            "tests/data/select/input_header1.tsv",
        ])
        .run_fail();

    assert!(stderr.contains("unknown field name `no_such_field`"));
}

#[test]
fn select_fields_exclude_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "--fields", "1", "--exclude", "2"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stderr.contains("--fields/-f and --exclude/-e cannot be used together"));
}

#[test]
fn select_missing_args() {
    let (_, stderr) = TvaCmd::new().args(&["select"]).stdin("a\tb\n").run_fail();

    assert!(stderr.contains("one of --fields/-f or --exclude/-e is required"));
}

#[test]
fn select_invalid_delimiter() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "-f", "1", "--delimiter", "TAB"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stderr.contains("delimiter must be a single character"));
}

#[test]
fn select_empty_selection() {
    let input = "a\tb\n1\t2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "--exclude", "1,2"])
        .stdin(input)
        .run();

    assert!(stdout.contains("\n\n")); // Two newlines for two rows
}

#[test]
fn select_invalid_field_spec() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "-f", "0"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn select_exclude_with_header() {
    let input = "h1\th2\th3\nv1\tv2\tv3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "--header", "--exclude", "2"])
        .stdin(input)
        .run();

    assert!(stdout.contains("h1\th3\nv1\tv3"));
}

#[test]
fn select_exclude_by_name_with_header() {
    let input = "h1\th2\th3\nv1\tv2\tv3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "--header", "--exclude", "h2"])
        .stdin(input)
        .run();

    assert!(stdout.contains("h1\th3\nv1\tv3"));
}
