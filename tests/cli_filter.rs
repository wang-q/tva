#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn filter_numeric_gt_basic() {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--gt", "value:10"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tvalue");
    assert_eq!(lines[1], "2\t15");
    assert_eq!(lines[2], "3\t20");
}

#[test]
fn filter_str_eq_basic() {
    let input = "id\tcolor\n1\tred\n2\tblue\n3\tred\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--str-eq", "color:red"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tcolor");
    assert_eq!(lines[1], "1\tred");
    assert_eq!(lines[2], "3\tred");
}

#[test]
fn filter_regex_basic() {
    let input = "id\tname\n1\talice\n2\tbob\n3\talex\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--regex", "name:^al"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "1\talice");
    assert_eq!(lines[2], "3\talex");
}

#[test]
fn filter_not_regex_basic() {
    let input = "id\tname\n1\talice\n2\tbob\n3\talex\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--not-regex", "name:^al"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "2\tbob");
}

#[test]
fn filter_char_len_gt_basic() {
    let input = "id\tname\n1\ta\n2\tabcd\n3\txyz\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--char-len-gt", "name:3"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "2\tabcd");
}

#[test]
fn filter_byte_len_lt_basic() {
    let input = "id\tname\n1\ta\n2\tabcd\n3\txyz\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--byte-len-lt", "name:4"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "1\ta");
    assert_eq!(lines[2], "3\txyz");
}

#[test]
fn filter_istr_not_in_fld_basic() {
    let input = "id\tdesc\n1\tHelloWorld\n2\tfoo\n3\tHELLO\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--istr-not-in-fld", "desc:hello"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "id\tdesc");
    assert_eq!(lines[1], "2\tfoo");
}

#[test]
fn filter_count_and_invert() {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter", "--header", "--gt", "value:10", "--invert", "--count",
        ])
        .stdin(input)
        .run();

    assert!(stdout.trim().starts_with("1"));
}

#[test]
fn filter_invalid_field_list_reports_error() {
    let input = "id\tvalue\n1\t5\n";
    let (_, stderr) = TvaCmd::new()
        .args(&["filter", "--header", "--gt", "0:10"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("tva filter:"));
}

#[test]
fn filter_is_numeric_basic() {
    let input = "id\tvalue\n1\t5\n2\tfoo\n3\t10.5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--is-numeric", "value"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tvalue");
    assert_eq!(lines[1], "1\t5");
    assert_eq!(lines[2], "3\t10.5");
}

#[test]
fn filter_ff_eq_basic() {
    let input = "id\ta\tb\n1\t5\t5\n2\t3\t4\n3\t7\t7\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--ff-eq", "a:b"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\ta\tb");
    assert_eq!(lines[1], "1\t5\t5");
    assert_eq!(lines[2], "3\t7\t7");
}

#[test]
fn filter_ff_str_ne_basic() {
    let input = "id\tx\ty\n1\tfoo\tfoo\n2\tbar\tbaz\n3\tHELLO\thello\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--ff-str-ne", "x:y"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tx\ty");
    assert_eq!(lines[1], "2\tbar\tbaz");
    assert_eq!(lines[2], "3\tHELLO\thello");
}

#[test]
fn filter_ff_absdiff_le_basic() {
    let input = "id\ta\tb\n1\t1.0\t1.2\n2\t5.0\t5.6\n3\t10.0\t10.1\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--ff-absdiff-le", "a:b:0.2"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\ta\tb");
    assert_eq!(lines[1], "1\t1.0\t1.2"); // |1.0-1.2| = 0.2
    assert_eq!(lines[2], "3\t10.0\t10.1"); // |10.0-10.1| = 0.1
}

#[test]
fn filter_ff_reldiff_gt_basic() {
    let input = "id\ta\tb\n1\t10\t12\n2\t100\t110\n3\t50\t55\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--ff-reldiff-gt", "a:b:0.09"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "id\ta\tb");
    assert_eq!(lines[1], "1\t10\t12"); // |2|/10 = 0.2 > 0.09
    assert_eq!(lines[2], "2\t100\t110"); // |10|/100 = 0.1 > 0.09
    assert_eq!(lines[3], "3\t50\t55"); // |5|/50 = 0.1 > 0.09
}

#[test]
fn filter_label_basic() {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t8\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--gt",
            "value:10",
            "--label",
            "pass",
            "--label-values",
            "Y:N",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "id\tvalue\tpass");
    assert_eq!(lines[1], "1\t5\tN");
    assert_eq!(lines[2], "2\t15\tY");
    assert_eq!(lines[3], "3\t8\tN");
}
