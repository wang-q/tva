use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn split_lines_per_file_basic() -> anyhow::Result<()> {
    let input: String = (1..=10).map(|i| format!("{}\n", i)).collect();

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("split")
        .arg("--lines-per-file")
        .arg("3")
        .arg("--dir")
        .arg(dir_str)
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)?
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

    Ok(())
}

#[test]
fn split_lines_per_file_with_header() -> anyhow::Result<()> {
    let input = "h\n1\n2\n3\n4\n";

    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("split")
        .arg("--lines-per-file")
        .arg("2")
        .arg("--header-in-out")
        .arg("--dir")
        .arg(dir_str)
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)?
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 2);

    for path in files {
        let contents = fs::read_to_string(&path)?;
        let mut lines = contents.lines();
        let header = lines.next().unwrap_or("");
        assert_eq!(header, "h");
        let data: Vec<&str> = lines.collect();
        assert!(data.len() <= 2);
    }

    Ok(())
}

#[test]
fn split_random_static_seed_reproducible() -> anyhow::Result<()> {
    let input: String = (1..=20).map(|i| format!("{}\n", i)).collect();

    let dir1 = tempdir().unwrap();
    let dir1_path: PathBuf = dir1.path().to_path_buf();
    let dir1_str = dir1_path.to_str().unwrap();

    let mut cmd1 = cargo_bin_cmd!("tva");
    let out1 = cmd1
        .arg("split")
        .arg("--num-files")
        .arg("3")
        .arg("--static-seed")
        .arg("--dir")
        .arg(dir1_str)
        .write_stdin(input.clone())
        .output()
        .unwrap();
    assert!(out1.status.success());

    let mut files1: Vec<PathBuf> = fs::read_dir(&dir1_path)?
        .map(|e| e.unwrap().path())
        .collect();
    files1.sort();

    let dir2 = tempdir().unwrap();
    let dir2_path: PathBuf = dir2.path().to_path_buf();
    let dir2_str = dir2_path.to_str().unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let out2 = cmd2
        .arg("split")
        .arg("--num-files")
        .arg("3")
        .arg("--static-seed")
        .arg("--dir")
        .arg(dir2_str)
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(out2.status.success());

    let mut files2: Vec<PathBuf> = fs::read_dir(&dir2_path)?
        .map(|e| e.unwrap().path())
        .collect();
    files2.sort();

    assert_eq!(files1.len(), files2.len());

    for (p1, p2) in files1.iter().zip(files2.iter()) {
        let c1 = fs::read_to_string(p1)?;
        let c2 = fs::read_to_string(p2)?;
        assert_eq!(c1, c2);
    }

    Ok(())
}

#[test]
fn split_random_by_key_groups_keys_together() -> anyhow::Result<()> {
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

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("split")
        .arg("--num-files")
        .arg("3")
        .arg("--key-fields")
        .arg("1")
        .arg("--dir")
        .arg(dir_str)
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)?
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    let mut key_to_bucket: HashMap<String, usize> = HashMap::new();

    for (bucket_idx, path) in files.iter().enumerate() {
        let contents = fs::read_to_string(path)?;
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

    Ok(())
}

#[test]
fn split_key_fields_invalid_field_list_reports_error() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .arg("--num-files")
        .arg("2")
        .arg("--key-fields")
        .arg("0")
        .write_stdin("a\t1\nb\t2\n");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("tva split:"));

    Ok(())
}

#[test]
fn split_requires_mode() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split");
    cmd.assert().failure().stderr(predicate::str::contains(
        "either --lines-per-file/-l or --num-files/-n must be specified",
    ));

    Ok(())
}

#[test]
fn split_rejects_conflicting_modes() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("split")
        .arg("--lines-per-file")
        .arg("10")
        .arg("--num-files")
        .arg("3")
        .write_stdin("1\n2\n3\n");
    cmd.assert().failure().stderr(predicate::str::contains(
        "tva split: --lines-per-file/-l cannot be used with --num-files/-n",
    ));

    Ok(())
}

#[test]
fn split_lines_per_file_from_input1x5() -> anyhow::Result<()> {
    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("split")
        .arg("--lines-per-file")
        .arg("3")
        .arg("--dir")
        .arg(dir_str)
        .arg("tests/data/split/input1x5.txt")
        .output()
        .unwrap();

    assert!(output.status.success());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)?
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 2);

    let contents0 = fs::read_to_string(&files[0])?;
    let contents1 = fs::read_to_string(&files[1])?;

    assert_eq!(
        contents0,
        "input1x5.txt: line 1\ninput1x5.txt: line 2\ninput1x5.txt: line 3\n"
    );
    assert_eq!(contents1, "input1x5.txt: line 4\ninput1x5.txt: line 5\n\n");

    Ok(())
}

#[test]
fn split_random_by_key_on_input4x18_groups_keys_together() -> anyhow::Result<()> {
    let dir = tempdir().unwrap();
    let dir_path: PathBuf = dir.path().to_path_buf();
    let dir_str = dir_path.to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("split")
        .arg("--num-files")
        .arg("3")
        .arg("--key-fields")
        .arg("1")
        .arg("--header-in-out")
        .arg("--dir")
        .arg(dir_str)
        .arg("tests/data/split/input4x18.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)?
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    let mut key_to_bucket: HashMap<String, usize> = HashMap::new();

    for (bucket_idx, path) in files.iter().enumerate() {
        let contents = fs::read_to_string(path)?;
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

    Ok(())
}
