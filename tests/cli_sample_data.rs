#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use std::path::PathBuf;

// Helper to create temp file with content
fn create_test_file(name: &str, content: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("tva_test_sample");
    fs::create_dir_all(&path).unwrap();
    path.push(name);
    fs::write(&path, content).unwrap();
    path
}

// Data sets from tsv-sample.d
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
    
    // We check that output is valid TSV, has header, and weights are respected statistically 
    // (though with fixed seed we get deterministic output).
    // Note: tva uses a different PRNG than tsv-utils (D language), so we cannot match exact output lines.
    // Instead we verify structural correctness and that weighted sampling logic ran.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 11); // Header + 10 rows
    assert!(lines[0].starts_with("random_value\tline\tweight"));
    
    // Verify all original lines are present (shuffle)
    let mut ids: Vec<&str> = lines[1..].iter().map(|l| l.split('\t').nth(1).unwrap()).collect();
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
    
    // Check random values are descending (weighted shuffle implementation detail)
    let random_vals: Vec<f64> = lines[1..]
        .iter()
        .map(|l| l.split('\t').next().unwrap().parse::<f64>().unwrap())
        .collect();
        
    for i in 0..random_vals.len() - 1 {
        assert!(random_vals[i] >= random_vals[i+1], "Random values should be descending in weighted shuffle output");
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
    // Key field 2 is "Shape"
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
    // Header should be present
    assert_eq!(lines[0], "ID\tShape\tColor\tSize\tWeight");
    
    // Verify distinct property: all rows with same Shape are either present or absent together
    let mut shapes_present = std::collections::HashSet::new();
    let mut shapes_absent = std::collections::HashSet::new();
    
    // All possible shapes in data5x25
    let all_shapes = vec!["circle", "square", "ellipse", "triangle"];
    
    for line in &lines[1..] {
        let shape = line.split('\t').nth(1).unwrap();
        shapes_present.insert(shape);
    }
    
    for shape in all_shapes {
        if !shapes_present.contains(shape) {
            shapes_absent.insert(shape);
        }
    }
    
    // Now verify against original data that no "absent" shape appears in output
    // (This is tautological here, but logic check)
    // More importantly: verify that for a present shape, ALL its instances are there?
    // Distinct sampling includes ALL records for selected keys.
    
    // Let's count expected instances for each shape in original data
    let mut shape_counts = std::collections::HashMap::new();
    for line in DATA_5X25.lines().skip(1) {
        let shape = line.split('\t').nth(1).unwrap();
        *shape_counts.entry(shape).or_insert(0) += 1;
    }
    
    // Count instances in output
    let mut output_shape_counts = std::collections::HashMap::new();
    for line in &lines[1..] {
        let shape = line.split('\t').nth(1).unwrap();
        *output_shape_counts.entry(shape).or_insert(0) += 1;
    }
    
    for shape in shapes_present {
        assert_eq!(output_shape_counts.get(shape), shape_counts.get(shape), 
            "All instances of shape {} should be present", shape);
    }
}

#[test]
fn sample_data5x25_distinct_k2_k4_p20() {
    let fpath = create_test_file("data5x25_k2k4.tsv", DATA_5X25);
    // Key fields 2 (Shape) and 4 (Size)
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
    
    // Verify key consistency
    let mut keys_present = std::collections::HashSet::new();
    
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split('\t').collect();
        let key = format!("{}-{}", cols[1], cols[3]);
        keys_present.insert(key);
    }
    
    // Check counts
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
        assert_eq!(key_counts_out.get(&key), key_counts_orig.get(&key),
             "All instances of key {} should be present", key);
    }
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
    
    // Verify key consistency for single column file
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
        assert_eq!(key_counts_out.get(key), key_counts_orig.get(key),
             "All instances of key {} should be present", key);
    }
}

#[test]
fn sample_data5x25_distinct_range_keys() {
    let fpath = create_test_file("data5x25_range.tsv", DATA_5X25);
    // Key fields 2-4 (Shape, Color, Size)
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
    
    // Check if output is consistent
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
        assert_eq!(key_counts_out.get(&key), key_counts_orig.get(&key),
             "All instances of key {} should be present", key);
    }
}
