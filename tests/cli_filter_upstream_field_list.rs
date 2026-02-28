#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_field_list_ge_range() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ge",
            "4-6:25",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
        "5\tad\t\t30\t35\t25\tbcdef\t40\t15\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_blank_2_3() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--blank",
            "2,3",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_empty_2_3_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--empty",
            "2,3,7",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_str_eq_4_6_0() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "4-6:0",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_str_in_fld_2_3_7_ab() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "2-3,7:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected =
        concat!("line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_str_in_fld_2_3_ab() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "2-3:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_str_not_in_fld_2_3_ab() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-not-in-fld",
            "2-3:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
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
}

#[test]
fn upstream_field_list_or_istr_eq_2_3_abc_upper() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--istr-eq",
            "2-3:abc",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_or_istr_eq_2_3_abc() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--istr-eq",
            "2-3:ABC",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_istr_in_fld_2_3_ab() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "2-3:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_or_regex_2_3_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--regex",
            "2-3,7:^.*b.*d$",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "4\taadd\taabdd\t10\t30\t15\tabd\t25\t25\n",
        "8\tbd\t\t10\t20\t40\tbcd\t15\t25\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_not_regex_2_3_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--not-regex",
            "2-3,7:^.*b.*d$",
            "tests/data/filter/input4.tsv",
        ])
        .run();
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
}

#[test]
fn upstream_field_list_or_iregex_7_3_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--iregex",
            "7,3,2:^.*b.*d$",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "4\taadd\taabdd\t10\t30\t15\tabd\t25\t25\n",
        "8\tbd\t\t10\t20\t40\tbcd\t15\t25\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
        "11\tAADD\tAABDD\t10\t30\t15\tABD\t25\t25\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_gt_range() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--gt",
            "4-6:25",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_eq_reverse_range() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--eq",
            "6-4,8:0",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_ne_ranges() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ne",
            "4-6,8-9:0",
            "tests/data/filter/input4.tsv",
        ])
        .run();
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
}

#[test]
fn upstream_field_list_or_eq_ranges() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--eq",
            "4-6,8-9:0",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "7\tbcf\tcc\t-20\t-50\t0\tabc\t0\t-5\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_field_list_lt_list() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--lt",
            "4,5:0",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "7\tbcf\tcc\t-20\t-50\t0\tabc\t0\t-5\n",
    );
    assert_eq!(stdout, expected);
}
