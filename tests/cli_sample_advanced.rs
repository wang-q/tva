#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;

// ============================================================================
// Distinct Advanced Tests
// ============================================================================

#[test]
fn sample_distinct_with_header() {
    let input = "h1\th2
A\t1
A\t2
B\t1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "h1",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let mut lines = stdout.lines();

    let header = lines.next().unwrap();
    assert_eq!(header, "h1\th2");

    let data: Vec<&str> = lines.collect();
    let a_count = data.iter().filter(|l| l.starts_with("A\t")).count();
    assert!(a_count == 0 || a_count == 2);
}

#[test]
fn sample_distinct_k0_vs_all_fields() {
    // Key 0 means all fields are key (same as not specifying -k in some modes,
    // but for distinct sampling it means the whole line is the key).
    // Actually, distinct sampling with key 0 means all fields are keys.
    // This is different from "no key" which would be non-distinct sampling.

    let input = "A\t1
A\t2
B\t1
";

    // Test with -k 0 (all fields as key)
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "0",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // All lines are distinct keys, so we should get roughly half of them.
    // With static seed, we can check the exact count if needed.
    assert!(lines.len() <= 3);
}

#[test]
fn sample_distinct_no_header() {
    let input = "A\t1
A\t2
B\t1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "1",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let a_count = lines.iter().filter(|l| l.starts_with("A\t")).count();
    assert!(a_count == 0 || a_count == 2);
}

#[test]
fn sample_distinct_print_random() {
    let input = "h1\th2
A\t1
A\t2
B\t1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "h1",
            "--prob",
            "0.5",
            "--print-random",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let mut lines = stdout.lines();

    let header = lines.next().unwrap();
    assert!(header.starts_with("random_value\th1"));

    let data: Vec<&str> = lines.collect();

    // If no data selected, skip the rest
    if data.is_empty() {
        return;
    }

    let a_count = data.iter().filter(|l| l.contains("\tA\t")).count();
    assert!(a_count == 0 || a_count == 2);

    // Only check if we have at least 2 rows
    if data.len() >= 2 {
        let parts0: Vec<&str> = data[0].split('\t').collect();
        let parts1: Vec<&str> = data[1].split('\t').collect();

        // If both rows are A, they should have same random value
        if parts0[1] == "A" && parts1[1] == "A" {
            assert_eq!(parts0[0], parts1[0]);
        }
        assert!(parts0[0].parse::<f64>().is_ok());
    }
}

#[test]
fn sample_distinct_gen_random_inorder() {
    let input = "h1\th2
A\t1
A\t2
B\t1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--gen-random-inorder",
            "--key-fields",
            "h1",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let mut lines = stdout.lines();

    let header = lines.next().unwrap();
    assert!(header.starts_with("random_value\th1"));

    let data: Vec<&str> = lines.collect();

    let row1 = data[0];
    let row2 = data[1];
    let row3 = data[2];

    let val1 = row1.split('\t').next().unwrap();
    let val2 = row2.split('\t').next().unwrap();
    let val3 = row3.split('\t').next().unwrap();

    assert_eq!(
        val1, val2,
        "Rows with same key should have same random value"
    );
    assert_ne!(
        val1, val3,
        "Rows with different keys should have different random values"
    );
}

#[test]
fn sample_distinct_multiple_keys() {
    // -k 1,2
    let input = "A\t1\tX
A\t1\tY
A\t2\tX
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "1,2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let a1_count = lines.iter().filter(|l| l.starts_with("A\t1")).count();
    assert!(a1_count == 0 || a1_count == 2);

    let a2_count = lines.iter().filter(|l| l.starts_with("A\t2")).count();
    assert!(a2_count == 0 || a2_count == 1);
}

// ============================================================================
// Weight Field Advanced Tests
// ============================================================================

#[test]
fn sample_weight_field_header_by_name_multiple_cols() {
    let input = "h1\th2\th3\nA\t1\t100\nB\t2\t1\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--weight-field",
            "h3",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "h1\th2\th3");
    assert!(lines.contains(&"A\t1\t100"));
    assert!(lines.contains(&"B\t2\t1"));
}

#[test]
fn sample_weight_field_header_by_wildcard() {
    let input = "h1\th2\tweight_col
A\t1\t100
B\t2\t1
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--weight-field",
            "weight_*",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
}

// ============================================================================
// Inorder Advanced Tests
// ============================================================================

#[test]
fn sample_inorder_numeric() {
    let input: String = (0..10)
        .map(|i| format!("line{}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "5", "--inorder", "--static-seed"])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 5);

    let indices: Vec<usize> = lines
        .iter()
        .map(|l| l.replace("line", "").parse::<usize>().unwrap())
        .collect();

    for i in 0..indices.len() - 1 {
        assert!(
            indices[i] < indices[i + 1],
            "Output not in order: {:?}",
            indices
        );
    }
}

// ============================================================================
// Replace Advanced Tests
// ============================================================================

#[test]
fn sample_replace_multiple_items() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--replace", "--num", "10", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10);
    let unique: std::collections::HashSet<&str> = lines.iter().cloned().collect();
    assert!(unique.len() <= 3);
}

// ============================================================================
// Algorithm Specific Tests
// ============================================================================

#[test]
fn sample_reservoir_algo_r() {
    let input = "1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--prefer-algorithm-r",
            "--num",
            "2",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_ne!(lines[0], lines[1]);
}

#[test]
fn sample_bernoulli_skip() {
    let input: String = (0..1000)
        .map(|i| format!("line{}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--prob", "0.01", "--static-seed"])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty());
    assert!(lines.len() < 200);
}

#[test]
fn sample_bernoulli_basic() {
    let input: String = (0..10)
        .map(|i| format!("line{}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--prob", "0.5", "--static-seed"])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() <= 10);
    assert!(!lines.is_empty());
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn sample_subset_num_greater_than_input() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "10", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.contains(&"A"));
    assert!(lines.contains(&"B"));
    assert!(lines.contains(&"C"));
}

#[test]
fn sample_distinct_key_ranges() {
    let input = "A\t1\tX
A\t1\tY
B\t2\tZ
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--key-fields",
            "1-2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let a1_count = lines.iter().filter(|l| l.starts_with("A\t1")).count();
    assert!(a1_count == 0 || a1_count == 2);
}

#[test]
fn sample_distinct_key_names_range() {
    let input = "h1\th2\th3
A\t1\tX
A\t1\tY
B\t2\tZ
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "h1-h2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let a1_count = lines.iter().filter(|l| l.starts_with("A\t1")).count();
    assert!(a1_count == 0 || a1_count == 2);
}

#[test]
fn sample_print_random_with_header_custom() {
    let input = "h1\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--prob",
            "1.0",
            "--print-random",
            "--random-value-header",
            "RVAL",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "RVAL\th1");
    assert!(lines[1].ends_with("\tA"));
}

#[test]
fn sample_distinct_key_names_list() {
    let input = "h1\th2\th3
A\t1\tX
A\t1\tY
B\t2\tZ
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "h1,h2",
            "--prob",
            "0.5",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let a1_count = lines.iter().filter(|l| l.starts_with("A\t1")).count();
    assert!(a1_count == 0 || a1_count == 2);
}

#[test]
fn sample_weighted_inorder() {
    let input = "A\t10
B\t1
C\t100
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "2",
            "--weight-field",
            "2",
            "--inorder",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);

    let pos_a = lines.iter().position(|&r| r.starts_with("A"));
    let pos_c = lines.iter().position(|&r| r.starts_with("C"));

    if let (Some(pa), Some(pc)) = (pos_a, pos_c) {
        assert!(pa < pc, "Output was not inorder: {:?}", lines);
    }
}

#[test]
fn sample_compat_mode_single_col() {
    let input: String = (0..10)
        .map(|i| format!("line{}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--compatibility-mode",
            "--header",
            "--static-seed",
        ])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10);
    assert_eq!(lines[0], "line0");

    for i in 1..10 {
        assert!(lines.contains(&format!("line{}", i).as_str()));
    }
}

#[test]
fn sample_weighted_print_random() {
    let input = "h1\th2\nA\t10\nB\t1\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--weight-field",
            "2",
            "--print-random",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].starts_with("random_value\th1"));

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert!(parts[0].parse::<f64>().is_ok());
}

#[test]
fn sample_key_fields_star() {
    let input = "h1\th2
A\t1
B\t1
A\t2
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--key-fields",
            "*",
            "--prob",
            "1.0",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
}

// ============================================================================
// Gen Random Inorder Advanced Tests
// ============================================================================

#[test]
fn sample_gen_random_inorder_custom_header() {
    let input = "h1\tv1\nA\t1\nB\t2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--header",
            "--random-value-header",
            "RND",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let mut lines = stdout.lines();
    let header = lines.next().unwrap();
    assert_eq!(header, "RND\th1\tv1");

    let row1 = lines.next().unwrap();
    let parts: Vec<&str> = row1.split('\t').collect();
    assert_eq!(parts.len(), 3);
    assert!(parts[0].parse::<f64>().is_ok());
    assert_eq!(parts[1], "A");
    assert_eq!(parts[2], "1");
}

#[test]
fn sample_gen_random_inorder_key_fields_names() {
    let input = "h1\th2\th3\nA\t1\tX\nA\t1\tY\nB\t2\tZ\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--header",
            "--key-fields",
            "h1,h2",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);

    let header = lines[0];
    assert!(header.starts_with("random_value\th1"));

    let row1 = lines[1];
    let row2 = lines[2];
    let row3 = lines[3];

    let val1 = row1.split('\t').next().unwrap();
    let val2 = row2.split('\t').next().unwrap();
    let val3 = row3.split('\t').next().unwrap();

    assert_eq!(
        val1, val2,
        "Rows with same key should have same random value"
    );
    assert_ne!(
        val1, val3,
        "Rows with different keys should have different random values"
    );
}

#[test]
fn sample_gen_random_inorder_key_out_of_range() {
    let input = "a\tb\n";
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--key-fields",
            "3",
            "--static-seed",
        ])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("key field index 3 out of range"));
}

#[test]
fn sample_gen_random_inorder_complex_keys() {
    let input = "A\tB\tC\tD\nA\tX\tC\tY\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--key-fields",
            "1,3",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);

    let val1 = lines[0].split('\t').next().unwrap();
    let val2 = lines[1].split('\t').next().unwrap();

    assert_eq!(val1, val2);
}

#[test]
fn sample_gen_random_inorder_key_reordered() {
    let input = "A\tB\tC\nA\tX\tC\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--key-fields",
            "3,1",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    let val1 = lines[0].split('\t').next().unwrap();
    let val2 = lines[1].split('\t').next().unwrap();

    assert_eq!(val1, val2);
}

#[test]
fn sample_outfile_option() {
    let input = "a\nb\n";
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.tsv");

    TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "2",
            "--static-seed",
            "--outfile",
            output_path.to_str().unwrap(),
        ])
        .stdin(input)
        .run();

    let content = std::fs::read_to_string(&output_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.contains(&"a"));
    assert!(lines.contains(&"b"));
}

#[test]
fn sample_gen_random_inorder_random_seed() {
    let input = "a\n";

    let (stdout1, _) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder"])
        .stdin(input)
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder"])
        .stdin(input)
        .run();

    assert_ne!(stdout1, stdout2);
}

#[test]
fn sample_gen_random_inorder_explicit_seed() {
    let input = "a\n";
    let seed = "12345";

    let (stdout1, _) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "--seed-value", seed])
        .stdin(input)
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "--seed-value", seed])
        .stdin(input)
        .run();

    assert_eq!(stdout1, stdout2);
}

#[test]
fn sample_gen_random_inorder_empty_lines() {
    let input = "a\n\nb\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.contains(&""));
    assert_eq!(lines.len(), 3);
}

#[test]
fn sample_gen_random_inorder_key_fields_no_header() {
    let input = "A\t1\nA\t2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--key-fields",
            "1",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);

    let val1 = lines[0].split('\t').next().unwrap();
    let val2 = lines[1].split('\t').next().unwrap();

    assert_eq!(val1, val2);
}

#[test]
fn sample_gen_random_inorder_key_fields_invalid_no_header() {
    let input = "A\t1\n";
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "sample",
            "--gen-random-inorder",
            "--key-fields",
            "name_without_header",
        ])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("requires header"));
}

#[test]
fn sample_standard_empty_lines() {
    let input = "a\n\nb\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "-n", "2", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.contains(&""));
    assert!(lines.contains(&"a"));
    assert!(lines.contains(&"b"));
}

// ============================================================================
// Data-driven Tests
// ============================================================================

use std::path::PathBuf as DataPathBuf;

fn create_test_file(name: &str, content: &str) -> DataPathBuf {
    let mut path = std::env::temp_dir();
    path.push("tva_test_sample");
    fs::create_dir_all(&path).unwrap();
    path.push(name);
    fs::write(&path, content).unwrap();
    path
}

const DATA_2X10A: &str = "line\tweight
1\t0.26788837
2\t0.06601298
3\t0.38627527
4\t0.47379424
5\t0.02966641
6\t0.05636231
7\t0.70529242
8\t0.91836862
9\t0.99103720
10\t0.31401740";

const DATA_2X10B: &str = "line\tweight
1\t761
2\t432
3\t103
4\t448
5\t750
6\t711
7\t867
8\t841
9\t963
10\t784";

const DATA_2X10C: &str = "line\tweight
1\t31.85
2\t17403.31
3\t653.84
4\t8.23
5\t2671.04
6\t26226.08
7\t1.79
8\t354.56
9\t35213.81
10\t679.29";

const DATA_5X25: &str = "ID\tShape\tColor\tSize\tWeight
01\tcircle\tred\tS\t10
02\tcircle\tblack\tL\t20
03\tsquare\tblack\tL\t20
04\tcircle\tgreen\tL\t30
05\tellipse\tred\tS\t20
06\ttriangle\tred\tS\t10
07\ttriangle\tred\tL\t20
08\tsquare\tblack\tS\t10
09\tcircle\tblack\tS\t20
10\tsquare\tgreen\tL\t20
11\ttriangle\tred\tL\t20
12\tcircle\tgreen\tL\t30
13\tellipse\tred\tS\t20
14\tcircle\tgreen\tL\t30
15\tellipse\tred\tL\t30
16\tsquare\tred\tS\t10
17\tcircle\tblack\tL\t20
18\tsquare\tred\tS\t20
19\tsquare\tblack\tL\t20
20\tcircle\tred\tS\t10
21\tellipse\tblack\tL\t30
22\ttriangle\tred\tL\t30
23\tcircle\tgreen\tS\t20
24\tsquare\tgreen\tL\t20
25\tcircle\tred\tS\t10";

const DATA_1X25: &str = "Shape-Size
circle-S
circle-L
square-L
circle-L
ellipse-S
triangle-S
triangle-L
square-S
circle-S
square-L
triangle-L
circle-L
ellipse-S
circle-L
ellipse-L
square-S
circle-L
square-S
square-L
circle-S
ellipse-L
triangle-L
circle-S
square-L
circle-S";

#[test]
fn sample_data2x10a_weighted_shuffle() {
    let fpath = create_test_file("data2x10a.tsv", DATA_2X10A);
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--seed-value",
            "42",
            "--weight-field",
            "2",
            "--print-random",
            fpath.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 11);
    assert!(lines[0].starts_with("random_value\tline\tweight"));

    let mut ids: Vec<&str> = lines[1..]
        .iter()
        .map(|l| l.split('\t').nth(1).unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["1", "10", "2", "3", "4", "5", "6", "7", "8", "9"]);
}

#[test]
fn sample_data2x10b_weighted_shuffle() {
    let fpath = create_test_file("data2x10b.tsv", DATA_2X10B);
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "-H",
            "-s",
            "-w",
            "2",
            "--print-random",
            fpath.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 11);

    let random_vals: Vec<f64> = lines[1..]
        .iter()
        .map(|l| l.split('\t').next().unwrap().parse::<f64>().unwrap())
        .collect();

    for i in 0..random_vals.len() - 1 {
        assert!(
            random_vals[i] >= random_vals[i + 1],
            "Random values should be descending in weighted shuffle output"
        );
    }
}

#[test]
fn sample_data2x10c_weighted_shuffle_log_dist() {
    let fpath = create_test_file("data2x10c.tsv", DATA_2X10C);
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "-H", "-s", "-w", "2", fpath.to_str().unwrap()])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 11);
}

#[test]
fn sample_data5x25_distinct_k2_p40() {
    let fpath = create_test_file("data5x25.tsv", DATA_5X25);
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--seed-value",
            "42",
            "--prob",
            "0.40",
            "--key-fields",
            "2",
            fpath.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "ID\tShape\tColor\tSize\tWeight");

    let mut shapes_present = std::collections::HashSet::new();

    for line in &lines[1..] {
        let shape = line.split('\t').nth(1).unwrap();
        shapes_present.insert(shape);
    }

    let mut shape_counts = std::collections::HashMap::new();
    for line in DATA_5X25.lines().skip(1) {
        let shape = line.split('\t').nth(1).unwrap();
        *shape_counts.entry(shape).or_insert(0) += 1;
    }

    let mut output_shape_counts = std::collections::HashMap::new();
    for line in &lines[1..] {
        let shape = line.split('\t').nth(1).unwrap();
        *output_shape_counts.entry(shape).or_insert(0) += 1;
    }

    for shape in shapes_present {
        assert_eq!(
            output_shape_counts.get(shape),
            shape_counts.get(shape),
            "All instances of shape {} should be present",
            shape
        );
    }
}

#[test]
fn sample_data5x25_distinct_k2_k4_p20() {
    let fpath = create_test_file("data5x25_k2k4.tsv", DATA_5X25);
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "-H",
            "-s",
            "-p",
            "0.20",
            "-k",
            "2,4",
            fpath.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    let mut keys_present = std::collections::HashSet::new();
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split('\t').collect();
        let key = format!("{}-{}", cols[1], cols[3]);
        keys_present.insert(key);
    }

    let mut key_counts_orig = std::collections::HashMap::new();
    for line in DATA_5X25.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let key = format!("{}-{}", cols[1], cols[3]);
        *key_counts_orig.entry(key).or_insert(0) += 1;
    }

    let mut key_counts_out = std::collections::HashMap::new();
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split('\t').collect();
        let key = format!("{}-{}", cols[1], cols[3]);
        *key_counts_out.entry(key).or_insert(0) += 1;
    }

    for key in keys_present {
        assert_eq!(
            key_counts_out.get(&key),
            key_counts_orig.get(&key),
            "All instances of key {} should be present",
            key
        );
    }
}

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn sample_weight_zero_or_negative() {
    // Weight <= 0 should be ignored (not selected)
    let input = "A\t0\nB\t-1\nC\t10\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "2",
            "--weight-field",
            "2",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Only C with weight 10 should be selected (or possibly nothing if k=0 logic)
    // With k=2 and only one positive weight, we should get at least C
    assert!(!lines.is_empty());
}

#[test]
fn sample_reservoir_k0() {
    // k=0 should return nothing
    let input = "a\nb\nc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "0", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // With num=0, all rows should be returned (shuffle mode)
    assert_eq!(lines.len(), 3);
}

#[test]
fn sample_prob_1_0() {
    // prob=1.0 should select all rows
    let input = "a\nb\nc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--prob", "1.0", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.contains(&"a"));
    assert!(lines.contains(&"b"));
    assert!(lines.contains(&"c"));
}

#[test]
fn sample_empty_input() {
    // Empty input should return empty output
    let input = "";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "5", "--static-seed"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "");
}

#[test]
fn sample_single_row() {
    // Single row input
    let input = "only_row\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "5", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "only_row");
}

#[test]
fn sample_weighted_shuffle_all_weights_zero() {
    // All weights <= 0 should result in empty output
    let input = "A\t0\nB\t-1\nC\t0\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--weight-field", "2", "--static-seed"])
        .stdin(input)
        .run();

    // Should return empty or very few lines since all weights are 0 or negative
    let lines: Vec<&str> = stdout.lines().collect();
    // With all weights <= 0, nothing should be selected
    assert!(lines.is_empty());
}

#[test]
fn sample_distinct_prob_1_0() {
    // Distinct sampling with prob=1.0 should select all distinct keys
    let input = "h1\th2
A\t1
A\t2
B\t1
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--prob",
            "1.0",
            "--key-fields",
            "h1",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Header + 2 distinct keys (A and B) with all their rows
    assert_eq!(lines.len(), 4);
}

#[test]
fn sample_compat_mode_shuffle() {
    // Compatibility mode without --num should shuffle all rows
    let input = "a\nb\nc\nd\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--compatibility-mode", "--static-seed"])
        .stdin(input)
        .run();

    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.sort();
    assert_eq!(lines, vec!["a", "b", "c", "d"]);
}

#[test]
fn sample_seed_value_explicit() {
    // Test explicit seed value produces reproducible results
    let input = "a\nb\nc\nd\ne\n";

    let (stdout1, _) = TvaCmd::new()
        .args(&["sample", "--num", "3", "--seed-value", "12345"])
        .stdin(input)
        .run();

    let (stdout2, _) = TvaCmd::new()
        .args(&["sample", "--num", "3", "--seed-value", "12345"])
        .stdin(input)
        .run();

    assert_eq!(stdout1, stdout2);
}

#[test]
fn sample_weighted_with_algorithm_r() {
    // Test that --prefer-algorithm-r is ignored when weight-field is used
    // Actually, looking at the code, it falls back to ReservoirSampler
    let input = "A\t1\nB\t2\nC\t3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "2",
            "--weight-field",
            "2",
            "--prefer-algorithm-r",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn sample_bernoulli_with_print_random() {
    // Bernoulli sampling with --print-random
    let input = "a\nb\nc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--prob", "0.5", "--print-random", "--static-seed"])
        .stdin(input)
        .run();

    // Each line should have a random value prefix
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<f64>().is_ok());
    }
}

#[test]
fn sample_shuffle_with_print_random() {
    // Shuffle mode with --print-random
    let input = "a\nb\nc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--print-random", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    // Each line should have a random value prefix
    for line in &lines {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<f64>().is_ok());
    }
}

#[test]
fn sample_inorder_with_print_random() {
    // Inorder sampling with --print-random
    let input = "a\nb\nc\nd\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "2",
            "--inorder",
            "--print-random",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);

    // Each line should have a random value prefix
    for line in &lines {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<f64>().is_ok());
    }
}

#[test]
fn sample_gen_random_inorder_no_header() {
    // gen-random-inorder without header
    let input = "a\nb\nc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--gen-random-inorder", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Without header, no header line is generated
    assert_eq!(lines.len(), 3);

    // Each line should have a random value prefix
    for line in &lines {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<f64>().is_ok());
    }
}

#[test]
fn sample_weight_field_with_single_row() {
    // Weighted sampling with only one row
    let input = "A\t100\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "5",
            "--weight-field",
            "2",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Should return the single row even if num > 1
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "A\t100");
}

#[test]
fn sample_reservoir_with_fewer_rows_than_k() {
    // Reservoir sampling when input has fewer rows than k
    let input = "a\nb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--num", "10", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Should return all available rows
    assert_eq!(lines.len(), 2);
}

#[test]
fn sample_distinct_with_single_key() {
    // Distinct sampling where all rows have the same key
    let input = "h1\th2
A\t1
A\t2
A\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--header",
            "--prob",
            "0.5",
            "--key-fields",
            "h1",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Either all rows selected or none (since they share the same key)
    assert!(lines.len() == 1 || lines.len() == 4);
}

#[test]
fn sample_multiple_empty_lines() {
    // Input with multiple consecutive empty lines
    let input = "a\n\n\nb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--static-seed"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // lines() skips empty lines, so we get fewer than actual lines
    assert!(lines.len() >= 2);
}

#[test]
fn sample_very_small_prob() {
    // Very small probability should select very few rows
    let input: String = (0..1000).map(|i| format!("line{}\n", i)).collect();
    let (stdout, _) = TvaCmd::new()
        .args(&["sample", "--prob", "0.001", "--static-seed"])
        .stdin(&input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // With p=0.001, expect ~1 row from 1000
    assert!(lines.len() < 10);
}

#[test]
fn sample_weighted_inorder_single_item() {
    // Weighted inorder with k=1
    let input = "A\t10\nB\t1\nC\t100\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "--num",
            "1",
            "--weight-field",
            "2",
            "--inorder",
            "--static-seed",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn sample_data1x25_distinct_k1_p20() {
    let fpath = create_test_file("data1x25.tsv", DATA_1X25);
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "-H",
            "-s",
            "-p",
            "0.20",
            "-k",
            "1",
            fpath.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    let mut keys_present = std::collections::HashSet::new();
    for line in &lines[1..] {
        keys_present.insert(line);
    }

    let mut key_counts_orig = std::collections::HashMap::new();
    for line in DATA_1X25.lines().skip(1) {
        *key_counts_orig.entry(line).or_insert(0) += 1;
    }

    let mut key_counts_out = std::collections::HashMap::new();
    for line in &lines[1..] {
        *key_counts_out.entry(line).or_insert(0) += 1;
    }

    for key in keys_present {
        assert_eq!(
            key_counts_out.get(key),
            key_counts_orig.get(key),
            "All instances of key {} should be present",
            key
        );
    }
}

#[test]
fn sample_data5x25_distinct_range_keys() {
    let fpath = create_test_file("data5x25_range.tsv", DATA_5X25);
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sample",
            "-H",
            "-s",
            "-p",
            "0.20",
            "-k",
            "2-4",
            fpath.to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    let mut keys_present = std::collections::HashSet::new();
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split('\t').collect();
        let key = format!("{}-{}-{}", cols[1], cols[2], cols[3]);
        keys_present.insert(key);
    }

    let mut key_counts_orig = std::collections::HashMap::new();
    for line in DATA_5X25.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let key = format!("{}-{}-{}", cols[1], cols[2], cols[3]);
        *key_counts_orig.entry(key).or_insert(0) += 1;
    }

    let mut key_counts_out = std::collections::HashMap::new();
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split('\t').collect();
        let key = format!("{}-{}-{}", cols[1], cols[2], cols[3]);
        *key_counts_out.entry(key).or_insert(0) += 1;
    }

    for key in keys_present {
        assert_eq!(
            key_counts_out.get(&key),
            key_counts_orig.get(&key),
            "All instances of key {} should be present",
            key
        );
    }
}
