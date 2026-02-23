use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn sample_shuffle_basic() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);

    let mut lines: Vec<&str> = out_str.lines().collect();
    lines.sort();
    assert_eq!(lines, vec!["a", "b", "c", "d"]);

    Ok(())
}

#[test]
fn sample_num_limited() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = out_str.lines().collect();

    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_prob_keeps_subset() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--prob")
        .arg("0.5")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = out_str.lines().collect();

    assert!(lines.len() <= 4);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_header_preserved() -> anyhow::Result<()> {
    let input = "h1\n1\n2\n3\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--header")
        .arg("--num")
        .arg("2")
        .write_stdin(input)
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let mut lines = out_str.lines();

    let header = lines.next().unwrap();
    assert_eq!(header, "h1");

    let data: Vec<&str> = lines.collect();
    assert_eq!(data.len(), 2);
    for line in &data {
        assert!(["1", "2", "3"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_invalid_prob_rejected() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("sample").arg("--prob").arg("0.0");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid --prob/-p value"));

    Ok(())
}

#[test]
fn sample_static_seed_produces_reproducible_output() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";

    let mut cmd1 = cargo_bin_cmd!("tva");
    let out1 = cmd1
        .arg("sample")
        .arg("--num")
        .arg("3")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();
    let s1 = String::from_utf8(out1.stdout).unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let out2 = cmd2
        .arg("sample")
        .arg("--num")
        .arg("3")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();
    let s2 = String::from_utf8(out2.stdout).unwrap();

    assert_eq!(s1, s2);

    Ok(())
}

#[test]
fn sample_replace_requires_num() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--replace")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("requires --num/-n greater than 0"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn sample_replace_conflicts_with_prob() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--replace")
        .arg("--num")
        .arg("2")
        .arg("--prob")
        .arg("0.5")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_replace_basic() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("10")
        .arg("--replace")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    Ok(())
}

#[test]
fn sample_inorder_requires_num() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--inorder")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_inorder_conflicts_with_prob() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .arg("--prob")
        .arg("0.5")
        .arg("--inorder")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_inorder_conflicts_with_replace() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .arg("--replace")
        .arg("--inorder")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn sample_inorder_basic() -> anyhow::Result<()> {
    let input = "a\nb\nc\nd\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sample")
        .arg("--num")
        .arg("2")
        .arg("--inorder")
        .arg("--static-seed")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert!(["a", "b", "c", "d"].contains(line));
    }

    let pos = |s: &str| match s {
        "a" => 0,
        "b" => 1,
        "c" => 2,
        "d" => 3,
        _ => 10,
    };
    assert!(pos(lines[0]) < pos(lines[1]));

    Ok(())
}
