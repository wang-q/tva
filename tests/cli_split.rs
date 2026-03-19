#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use test_case::test_case;

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(&["split"], "either --lines-per-file/-l or --num-files/-n must be specified" ; "requires_mode")]
#[test_case(&["split", "-l", "10", "-n", "2"], "--lines-per-file/-l cannot be used with --num-files/-n" ; "lines_num_conflict")]
#[test_case(&["split", "-k", "1", "-l", "10"], "--key-fields/-k is only supported with --num-files/-n" ; "key_lines_conflict")]
#[test_case(&["split", "-k", "1"], "either --lines-per-file/-l or --num-files/-n must be specified" ; "key_no_num")]
fn split_error_cases(args: &[&str], expected_error: &str) {
    let (_, stderr) = TvaCmd::new().args(args).stdin("a\n").run_fail();
    assert!(stderr.contains(expected_error));
}

#[test]
fn split_key_fields_invalid_field_list_reports_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["split", "--num-files", "2", "--key-fields", "0"])
        .stdin("a\t1\nb\t2\n")
        .run_fail();

    assert!(stderr.contains("tva split:"));
}

#[test]
fn split_output_not_dir() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("file");
    fs::write(&file_path, "content").unwrap();

    let (_, stderr) = TvaCmd::new()
        .args(&["split", "-n", "2", "--dir", file_path.to_str().unwrap()])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("output path is not a directory"));
}

#[test]
fn split_file_exists_no_append() {
    let temp = tempdir().unwrap();
    let dir = temp.path();

    let file_path = dir.join("split-0.tsv");
    fs::write(&file_path, "existing").unwrap();

    let (_, stderr) = TvaCmd::new()
        .args(&[
            "split",
            "-n",
            "1",
            "--static-seed",
            "--dir",
            dir.to_str().unwrap(),
        ])
        .stdin("row1\n")
        .run_fail();

    assert!(stderr.contains("output file already exists"));
}

// ============================================================================
// Lines Per File Tests
// ============================================================================

#[test]
fn split_lines_per_file_basic() {
    let input: String = (1..=10).map(|i| format!("{}\n", i)).collect();

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["split", "--lines-per-file", "3", "--dir", dir_str])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 4);

    let counts: Vec<usize> = files
        .iter()
        .map(|path| {
            let contents = fs::read_to_string(path).unwrap();
            contents.lines().count()
        })
        .collect();

    assert_eq!(counts, vec![3, 3, 3, 1]);
}

#[test]
fn split_lines_per_file_with_header() {
    let input = "h\n1\n2\n3\n4\n";

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "2",
            "--header-in-out",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 2);

    for path in files {
        let contents = fs::read_to_string(&path).unwrap();
        let mut lines = contents.lines();
        let header = lines.next().unwrap_or("");
        assert_eq!(header, "h");
        let data: Vec<&str> = lines.collect();
        assert!(data.len() <= 2);
    }
}

#[test]
fn split_lines_per_file_from_input1x5() {
    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "3",
            "--dir",
            dir_str,
            "tests/data/split/input1x5.txt",
        ])
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 2);

    let contents0 = fs::read_to_string(&files[0]).unwrap();
    let contents1 = fs::read_to_string(&files[1]).unwrap();

    assert_eq!(
        contents0,
        "input1x5.txt: line 1\ninput1x5.txt: line 2\ninput1x5.txt: line 3\n"
    );
    assert_eq!(contents1, "input1x5.txt: line 4\ninput1x5.txt: line 5\n\n");
}

#[test]
fn split_header_in_only_skips_header() {
    let input = "Header\n1\n2\n3\n4\n";

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "2",
            "--header-in-only",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort_by_key(|p| p.file_name().unwrap().to_owned());

    assert_eq!(files.len(), 2);

    let c1 = fs::read_to_string(&files[0]).unwrap();
    assert_eq!(c1, "1\n2\n");

    let c2 = fs::read_to_string(&files[1]).unwrap();
    assert_eq!(c2, "3\n4\n");
}

// ============================================================================
// Random/Num Files Tests
// ============================================================================

#[test]
fn split_random_static_seed_reproducible() {
    let input: String = (1..=20).map(|i| format!("{}\n", i)).collect();

    let dir1 = tempdir().unwrap();
    let dir1_path: PathBuf = dir1.path().to_path_buf();
    let dir1_str = dir1_path.to_str().unwrap();

    let (out1, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "3",
            "--static-seed",
            "--dir",
            dir1_str,
        ])
        .stdin(input.clone())
        .run();
    assert!(out1.is_empty());

    let mut files1: Vec<PathBuf> = fs::read_dir(&dir1_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files1.sort();

    let dir2 = tempdir().unwrap();
    let dir2_path: PathBuf = dir2.path().to_path_buf();
    let dir2_str = dir2_path.to_str().unwrap();

    let (out2, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "3",
            "--static-seed",
            "--dir",
            dir2_str,
        ])
        .stdin(input)
        .run();
    assert!(out2.is_empty());

    let mut files2: Vec<PathBuf> = fs::read_dir(&dir2_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files2.sort();

    assert_eq!(files1.len(), files2.len());

    for (p1, p2) in files1.iter().zip(files2.iter()) {
        let c1 = fs::read_to_string(p1).unwrap();
        let c2 = fs::read_to_string(p2).unwrap();
        assert_eq!(c1, c2);
    }
}

#[test]
fn split_random_by_key_groups_keys_together() {
    let input = "\
a\t1
b\t2
a\t3
c\t4
b\t5
";

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "3",
            "--key-fields",
            "1",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    let mut key_to_bucket: HashMap<String, usize> = HashMap::new();

    for (bucket_idx, path) in files.iter().enumerate() {
        let contents = fs::read_to_string(path).unwrap();
        for line in contents.lines() {
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split('\t');
            let key = parts.next().unwrap_or("").to_string();
            if let Some(prev_bucket) = key_to_bucket.get(&key) {
                assert_eq!(*prev_bucket, bucket_idx);
            } else {
                key_to_bucket.insert(key, bucket_idx);
            }
        }
    }
}

#[test]
fn split_random_by_key_on_input4x18_groups_keys_together() {
    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "3",
            "--key-fields",
            "1",
            "--header-in-out",
            "--dir",
            dir_str,
            "tests/data/split/input4x18.tsv",
        ])
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    let mut key_to_bucket: HashMap<String, usize> = HashMap::new();

    for (bucket_idx, path) in files.iter().enumerate() {
        let contents = fs::read_to_string(path).unwrap();
        for (line_idx, line) in contents.lines().enumerate() {
            if line_idx == 0 {
                continue;
            }
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split('\t');
            let key = parts.next().unwrap_or("").to_string();
            if let Some(prev_bucket) = key_to_bucket.get(&key) {
                assert_eq!(*prev_bucket, bucket_idx);
            } else {
                key_to_bucket.insert(key, bucket_idx);
            }
        }
    }
}

#[test]
fn split_random_num_files_1() {
    let input = "1\n2\n3\n4\n5\n";

    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "1",
            "--static-seed",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let files: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    assert_eq!(files.len(), 1);

    let content = fs::read_to_string(&files[0]).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn split_max_open_files_lru() {
    let input = "1\n2\n3\n4\n5\n6\n";

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "3",
            "--max-open-files",
            "1",
            "--dir",
            dir_str,
            "--static-seed",
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    let mut all_lines = Vec::new();
    for path in files {
        let content = fs::read_to_string(path).unwrap();
        for line in content.lines() {
            all_lines.push(line.to_string());
        }
    }
    all_lines.sort();
    assert_eq!(all_lines, vec!["1", "2", "3", "4", "5", "6"]);
}

// ============================================================================
// Seed and Reproducibility Tests
// ============================================================================

#[test]
fn split_seed_value_reproducible() {
    let input: String = (1..=20).map(|i| format!("{}\n", i)).collect();

    let dir1 = tempdir().unwrap();
    let dir1_str = dir1.path().to_str().unwrap();

    let (out1, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "3",
            "--seed-value",
            "12345",
            "--dir",
            dir1_str,
        ])
        .stdin(input.clone())
        .run();
    assert!(out1.is_empty());

    let mut files1: Vec<PathBuf> = fs::read_dir(dir1.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files1.sort();

    let dir2 = tempdir().unwrap();
    let dir2_str = dir2.path().to_str().unwrap();

    let (out2, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "3",
            "--seed-value",
            "12345",
            "--dir",
            dir2_str,
        ])
        .stdin(input)
        .run();
    assert!(out2.is_empty());

    let mut files2: Vec<PathBuf> = fs::read_dir(dir2.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files2.sort();

    assert_eq!(files1.len(), files2.len());

    for (p1, p2) in files1.iter().zip(files2.iter()) {
        let c1 = fs::read_to_string(p1).unwrap();
        let c2 = fs::read_to_string(p2).unwrap();
        assert_eq!(c1, c2);
    }
}

// ============================================================================
// Append Mode Tests
// ============================================================================

#[test]
fn split_append_mode_correctness() {
    let input1 = "H\n1\n2\n";
    let input2 = "H\n3\n4\n";

    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "2",
            "--header-in-out",
            "--dir",
            dir_str,
            "--prefix",
            "part-",
            "--suffix",
            ".txt",
            "--digit-width",
            "1",
        ])
        .stdin(input1)
        .run();

    TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "2",
            "--header-in-out",
            "--dir",
            dir_str,
            "--prefix",
            "part-",
            "--suffix",
            ".txt",
            "--digit-width",
            "1",
            "--append",
        ])
        .stdin(input2)
        .run();

    let f1 = dir.path().join("part-0.txt");
    let c1 = fs::read_to_string(f1).unwrap();

    assert_eq!(c1, "H\n1\n2\n3\n4\n");
}

#[test]
fn split_random_with_append() {
    let input1 = "1\n2\n3\n";
    let input2 = "4\n5\n6\n";

    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "2",
            "--static-seed",
            "--dir",
            dir_str,
        ])
        .stdin(input1)
        .run();

    TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "2",
            "--static-seed",
            "--dir",
            dir_str,
            "--append",
        ])
        .stdin(input2)
        .run();

    let mut files: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 2);

    let mut all_lines = Vec::new();
    for path in files {
        let content = fs::read_to_string(path).unwrap();
        for line in content.lines() {
            all_lines.push(line.to_string());
        }
    }
    all_lines.sort();
    assert_eq!(all_lines, vec!["1", "2", "3", "4", "5", "6"]);
}

// ============================================================================
// Multi-File Input Tests
// ============================================================================

#[test]
fn split_multiple_files_with_headers() {
    let input1 = "H1\tH2\nA\t1\nB\t2\n";
    let input2 = "H1\tH2\nC\t3\nD\t4\n";

    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let in1 = dir_path.join("in1.tsv");
    let in2 = dir_path.join("in2.tsv");
    fs::write(&in1, input1).unwrap();
    fs::write(&in2, input2).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "2",
            "--header-in-out",
            "--dir",
            dir_path.to_str().unwrap(),
            in1.to_str().unwrap(),
            in2.to_str().unwrap(),
        ])
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("split-")
        })
        .collect();
    files.sort();

    assert_eq!(files.len(), 2);

    let c1 = fs::read_to_string(&files[0]).unwrap();
    assert_eq!(c1, "H1\tH2\nA\t1\nB\t2\n");

    let c2 = fs::read_to_string(&files[1]).unwrap();
    assert_eq!(c2, "H1\tH2\nC\t3\nD\t4\n");
}

// ============================================================================
// Delimiter and Key Tests
// ============================================================================

#[test]
fn split_delimiter_custom() {
    let input = "a,1\nb,1\nc,2\nd,2\n";

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "2",
            "--key-fields",
            "2",
            "--delimiter",
            ",",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    let mut key_map = HashMap::new();
    for (f_idx, path) in files.iter().enumerate() {
        let content = fs::read_to_string(path).unwrap();
        for line in content.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            let key = parts[1];
            if let Some(&prev_idx) = key_map.get(key) {
                assert_eq!(prev_idx, f_idx, "Key {} found in multiple files", key);
            } else {
                key_map.insert(key.to_string(), f_idx);
            }
        }
    }
}

#[test]
fn split_key_fields_with_empty_fields() {
    let input = "a\t\n\tb\nc\t\n";

    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--num-files",
            "2",
            "--key-fields",
            "1,2",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let files: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    assert!(!files.is_empty());
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test_case("", 0, 100 ; "empty_file")]
#[test_case("1\n2\n", 1, 100 ; "basic_lines_single_file")]
#[test_case("1\n2\n3\n4\n", 2, 2 ; "basic_lines_two_files")]
fn split_empty_and_basic(
    input: &str,
    expected_file_count: usize,
    lines_per_file: usize,
) {
    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["split", "-l", &lines_per_file.to_string(), "--dir", dir_str])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let count = fs::read_dir(dir.path()).unwrap().count();
    assert_eq!(count, expected_file_count);
}

#[test]
fn split_empty_lines_in_data() {
    let input = "line1\n\nline2\n\n\nline3\n";

    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["split", "--lines-per-file", "2", "--dir", dir_str])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let mut files: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    let mut total_lines = 0;
    for path in files {
        let content = fs::read_to_string(path).unwrap();
        total_lines += content.lines().count();
    }
    assert_eq!(total_lines, 6);
}

#[test]
fn split_creates_output_dir() {
    let input = "1\n2\n3\n";

    let dir = tempdir().unwrap();
    let nested_dir = dir.path().join("nested").join("output");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "2",
            "--dir",
            nested_dir.to_str().unwrap(),
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());
    assert!(nested_dir.exists());

    let files: Vec<PathBuf> = fs::read_dir(&nested_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files.len(), 2);
}

#[test]
fn split_numeric_filenames_padding() {
    let input = "1\n2\n";
    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    TvaCmd::new()
        .args(&["split", "-l", "1", "--digit-width", "3", "--dir", dir_str])
        .stdin(input)
        .run();

    let f1 = dir.path().join("split-000.tsv");
    let f2 = dir.path().join("split-001.tsv");

    assert!(f1.exists());
    assert!(f2.exists());
}

#[test]
fn split_prefix_and_suffix_custom() {
    let input = "1\n2\n";

    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "1",
            "--prefix",
            "part_",
            "--suffix",
            ".dat",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let f1 = dir.path().join("part_000.dat");
    let f2 = dir.path().join("part_001.dat");

    assert!(f1.exists());
    assert!(f2.exists());
}

#[test]
fn split_lines_with_header_and_empty_lines() {
    let input = "Header\n\ndata1\n\ndata2\n";

    let dir = tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "2",
            "--header-in-out",
            "--dir",
            dir_str,
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let files: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    assert!(!files.is_empty());

    let mut sorted_files = files;
    sorted_files.sort();
    let first_content = fs::read_to_string(&sorted_files[0]).unwrap();
    assert!(first_content.starts_with("Header"));
}
