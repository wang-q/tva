#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

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
fn split_key_fields_invalid_field_list_reports_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["split", "--num-files", "2", "--key-fields", "0"])
        .stdin("a\t1\nb\t2\n")
        .run_fail();

    assert!(stderr.contains("tva split:"));
}

#[test]
fn split_requires_mode() {
    let (_, stderr) = TvaCmd::new().args(&["split"]).run_fail();

    assert!(stderr.contains("either --lines-per-file/-l or --num-files/-n must be specified"));
}

#[test]
fn split_rejects_conflicting_modes() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "split",
            "--lines-per-file",
            "10",
            "--num-files",
            "3",
        ])
        .stdin("1\n2\n3\n")
        .run_fail();

    assert!(stderr.contains("tva split: --lines-per-file/-l cannot be used with --num-files/-n"));
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
fn split_missing_args() {
    let (_, stderr) = TvaCmd::new().args(&["split"]).stdin("a\n").run_fail();

    assert!(stderr.contains("either --lines-per-file/-l or --num-files/-n must be specified"));
}

#[test]
fn split_lines_num_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["split", "-l", "10", "-n", "2"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--lines-per-file/-l cannot be used with --num-files/-n"));
}

#[test]
fn split_key_lines_conflict() {
    let (_, stderr) = TvaCmd::new()
        .args(&["split", "-k", "1", "-l", "10"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("--key-fields/-k is only supported with --num-files/-n"));
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

    // Create a file that split would try to create: split-1.tsv
    let file_path = dir.join("split-1.tsv");
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

#[test]
fn split_key_no_num() {
    let (_, stderr) = TvaCmd::new()
        .args(&["split", "-k", "1"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("either --lines-per-file/-l or --num-files/-n must be specified"));
}
