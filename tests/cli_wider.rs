#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    write!(file, "{}", content).expect("failed to write to temp file");
    file
}

#[test]
fn wider_basic() {
    let input = "
ID\tname\tvalue
A\tcost\t10
A\tsize\t5
B\tcost\t20
B\tsize\t8
";
    let expected = "
ID\tcost\tsize
A\t10\t5
B\t20\t8
";
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "name", "--values-from", "value"])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_implicit_id_multi_col() {
    let input = "
A\tB\tkey\tval
1\tx\tk1\t10
1\tx\tk2\t20
2\ty\tk1\t30
";
    // Expected:
    // A  B  k1  k2
    // 1  x  10  20
    // 2  y  30
    let expected = "
A\tB\tk1\tk2
1\tx\t10\t20
2\ty\t30\t
";
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "key", "--values-from", "val"])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_names_sort() {
    let input = "
ID\tkey\tval
1\tb\t2
1\ta\t1
1\tc\t3
";
    let expected = "
ID\ta\tb\tc
1\t1\t2\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "key",
            "--values-from",
            "val",
            "--names-sort",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_custom_fill_string() {
    let input = "
ID\tkey\tval
1\ta\t1
2\tb\t2
";
    let expected = "
ID\ta\tb
1\t1\tmissing
2\tmissing\t2
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "key",
            "--values-from",
            "val",
            "--values-fill",
            "missing",
            "--names-sort",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_missing_values() {
    let input = "
ID\tname\tvalue
A\tcost\t10
B\tsize\t8
";
    let expected = "
ID\tcost\tsize
A\t10\t0
B\t0\t8
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--values-fill",
            "0",
            "--names-sort",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_explicit_id() {
    let input = "
ID\tDate\tname\tvalue
A\t2020\tcost\t10
A\t2021\tcost\t12
";
    let expected = "
ID\tcost
A\t12
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_doc_example_us_rent_income() {
    let expected = "
GEOID\tNAME\tincome\trent
01\tAlabama\t24476\t747
02\tAlaska\t32940\t1200
04\tArizona\t27517\t972
05\tArkansas\t23789\t709
06\tCalifornia\t29454\t1358
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "docs/data/us_rent_income.tsv",
            "--names-from",
            "variable",
            "--values-from",
            "estimate",
            "--id-cols",
            "GEOID,NAME",
        ])
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_multi_file_error() {
    let file1 = create_file("ID\tname\tvalue\nA\tcost\t10\n");
    let file2 = create_file("ID\tvalue\nB\t20\n");

    let (_, stderr) = TvaCmd::new()
        .args(&[
            "wider",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            "--names-from",
            "name",
            "--values-from",
            "value",
        ])
        .run_fail();

    assert!(stderr.contains("All files must have the same column structure"));
}

#[test]
fn wider_preserve_space() {
    let input = "
ID\tname\tvalue
A\tcost\t 
";
    let expected = "
ID\tcost
A\t 
";
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "name", "--values-from", "value"])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_datamash_scenarios() {
    // Scenario 1
    let input1 = "
ID\tKey\tVal
a\tx\t1
a\ty\t2
a\tx\t3
";
    let expected1 = "
ID\tx\ty
a\t3\t2
";
    let (stdout1, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
        ])
        .stdin(input1.trim())
        .run();

    assert_eq!(stdout1.trim(), expected1.trim());

    // Scenario 2
    let input2 = "
ID\tKey\tVal
a\tx\t1
a\ty\t2
b\tx\t3
";
    let expected2 = "
ID\tx\ty
a\t1\t2
b\t3\tXX
";
    let (stdout2, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--values-fill",
            "XX",
        ])
        .stdin(input2.trim())
        .run();

    assert_eq!(stdout2.trim(), expected2.trim());
}

#[test]
fn wider_aggregation_ops() {
    let input = "
ID\tname\tval
A\tX\t10
A\tX\t20
B\tY\t5
B\tY\t15
C\tZ\t100
";

    // 1. Test SUM
    let expected_sum = "
ID\tX\tY\tZ
A\t30\t\t
B\t\t20\t
C\t\t\t100
";
    let (stdout_sum, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "val",
            "--id-cols",
            "ID",
            "--op",
            "sum",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout_sum.trim(), expected_sum.trim());

    // 2. Test MEAN
    let expected_mean = "
ID\tX\tY\tZ
A\t15\t\t
B\t\t10\t
C\t\t\t100
";
    let (stdout_mean, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "val",
            "--id-cols",
            "ID",
            "--op",
            "mean",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout_mean.trim(), expected_mean.trim());

    // 3. Test COUNT
    let expected_count = "
ID\tX\tY\tZ
A\t2\t\t
B\t\t2\t
C\t\t\t1
";
    let (stdout_count, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--id-cols",
            "ID",
            "--op",
            "count",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout_count.trim(), expected_count.trim());
}

#[test]
fn wider_extended_stats() {
    let input = "
ID\tKey\tVal
A\tX\t1
A\tX\t3
A\tX\t5
B\tX\t2
B\tX\t2
B\tX\t8
";

    // 1. Min
    let (stdout_min, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "min",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_min.contains("A\t1"));
    assert!(stdout_min.contains("B\t2"));

    // Max
    let (stdout_max, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "max",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_max.contains("A\t5"));
    assert!(stdout_max.contains("B\t8"));

    // 2. Median
    let (stdout_median, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "median",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_median.contains("A\t3"));
    assert!(stdout_median.contains("B\t2"));

    // 3. Mode
    let (stdout_mode, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "mode",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_mode.contains("A\t1"));
    assert!(stdout_mode.contains("B\t2"));
}

#[test]
fn wider_first_last() {
    let input = "
ID\tKey\tVal
A\tX\tfirst_val
A\tX\tmiddle_val
A\tX\tlast_val
";

    // First
    let (stdout_first, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "first",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_first.contains("A\tfirst_val"));

    // Last
    let (stdout_last, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "last",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_last.contains("A\tlast_val"));
}

#[test]
fn wider_quartiles_iqr() {
    let input = "
ID\tKey\tVal
A\tX\t1
A\tX\t2
A\tX\t3
A\tX\t4
A\tX\t5
";

    // Q1
    let (stdout_q1, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "q1",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_q1.contains("A\t2"));

    // Q3
    let (stdout_q3, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "q3",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_q3.contains("A\t4"));

    // IQR
    let (stdout_iqr, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "iqr",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_iqr.contains("A\t2"));
}

#[test]
fn wider_advanced_math_stats() {
    let input = "
ID\tKey\tVal
A\tX\t2
A\tX\t8
";

    // GeoMean
    let (stdout_geo, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "geomean",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_geo.contains("A\t4"));

    // HarmMean
    let (stdout_harm, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "harmmean",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_harm.contains("A\t3.2"));

    // Variance
    let (stdout_var, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "variance",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_var.contains("A\t18"));

    // Stdev
    let (stdout_std, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "stdev",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_std.contains("A\t4.242"));

    // CV
    let (stdout_cv, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "cv",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_cv.contains("A\t0.848"));
}
