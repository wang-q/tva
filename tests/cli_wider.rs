use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn wider_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tname\tvalue")?;
    writeln!(file, "A\tcost\t10")?;
    writeln!(file, "A\tsize\t5")?;
    writeln!(file, "B\tcost\t20")?;
    writeln!(file, "B\tsize\t8")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .output()?;

    let expected = "ID\tcost\tsize\nA\t10\t5\nB\t20\t8\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_implicit_id_multi_col() -> anyhow::Result<()> {
    // Ref: tidyr test "non-pivoted cols are preserved"
    // If id-cols is not specified, all other cols are IDs.
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "A\tB\tkey\tval")?;
    writeln!(file, "1\tx\tk1\t10")?;
    writeln!(file, "1\tx\tk2\t20")?;
    writeln!(file, "2\ty\tk1\t30")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("key")
        .arg("--values-from")
        .arg("val")
        // No --id-cols, so A and B should be IDs
        .output()?;

    // Expected:
    // A  B  k1  k2
    // 1  x  10  20
    // 2  y  30
    let expected = "A\tB\tk1\tk2\n1\tx\t10\t20\n2\ty\t30\t\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_names_sort() -> anyhow::Result<()> {
    // Ref: tidyr test "can sort column names"
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tkey\tval")?;
    writeln!(file, "1\tb\t2")?;
    writeln!(file, "1\ta\t1")?;
    writeln!(file, "1\tc\t3")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("key")
        .arg("--values-from")
        .arg("val")
        .arg("--names-sort") // Should sort a, b, c
        .output()?;

    let expected = "ID\ta\tb\tc\n1\t1\t2\t3\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_custom_fill_string() -> anyhow::Result<()> {
    // Ref: tidyr test "can fill in missing cells"
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tkey\tval")?;
    writeln!(file, "1\ta\t1")?;
    writeln!(file, "2\tb\t2")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("key")
        .arg("--values-from")
        .arg("val")
        .arg("--values-fill")
        .arg("missing")
        .arg("--names-sort")
        .output()?;

    // Expected:
    // ID a       b
    // 1  1       missing
    // 2  missing 2
    let expected = "ID\ta\tb\n1\t1\tmissing\n2\tmissing\t2\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_missing_values() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tname\tvalue")?;
    writeln!(file, "A\tcost\t10")?;
    writeln!(file, "B\tsize\t8")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .arg("--values-fill")
        .arg("0")
        .arg("--names-sort")
        .output()?;

    let expected = "ID\tcost\tsize\nA\t10\t0\nB\t0\t8\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_explicit_id() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tDate\tname\tvalue")?;
    writeln!(file, "A\t2020\tcost\t10")?;
    writeln!(file, "A\t2021\tcost\t12")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .arg("--id-cols")
        .arg("ID")
        .output()?;

    // With explicit ID "ID", "Date" is ignored.
    // Row 1: ID=A, name=cost, value=10
    // Row 2: ID=A, name=cost, value=12
    // Row 2 overwrites Row 1 because ID and Name are same.
    let expected = "ID\tcost\nA\t12\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_doc_example_us_rent_income() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("wider")
        .arg("docs/data/us_rent_income.tsv")
        .arg("--names-from")
        .arg("variable")
        .arg("--values-from")
        .arg("estimate")
        .arg("--id-cols")
        .arg("GEOID,NAME")
        .output()?;

    let expected = "GEOID\tNAME\tincome\trent\n01\tAlabama\t24476\t747\n02\tAlaska\t32940\t1200\n04\tArizona\t27517\t972\n05\tArkansas\t23789\t709\n06\tCalifornia\t29454\t1358\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_multi_file_error() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file1 = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file1, "ID\tname\tvalue")?;
    writeln!(file1, "A\tcost\t10")?;

    let mut file2 = tempfile::NamedTempFile::new()?;
    // Only 2 columns, but first file had 3
    writeln!(file2, "ID\tvalue")?;
    writeln!(file2, "B\t20")?;

    let path1 = file1.path().to_str().unwrap();
    let path2 = file2.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path1)
        .arg(path2)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("All files must have the same column structure"));
    Ok(())
}

#[test]
fn wider_preserve_space() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tname\tvalue")?;
    // Value is a space
    writeln!(file, "A\tcost\t ")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .output()?;

    let expected = "ID\tcost\nA\t \n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_datamash_scenarios() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");

    // Scenario 1: Unsorted input with duplicates (mirrors Datamash in2/out2_last_unsorted)
    // datamash: crosstab 1,2 last 3
    // tva: wider --names-from 2 --values-from 3 --id-cols 1
    let mut file1 = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file1, "a\tx\t1")?;
    writeln!(file1, "a\ty\t2")?;
    writeln!(file1, "a\tx\t3")?;
    let path1 = file1.path().to_str().unwrap();

    let output1 = cmd
        .arg("wider")
        .arg(path1)
        .arg("--names-from")
        .arg("2")
        .arg("--values-from")
        .arg("3")
        .arg("--id-cols")
        .arg("1")
        .output()?;

    // Expected: a, x=3 (last wins), y=2
    // tva outputs ID column name first (which is "a" from the header? wait, input has headers?)
    // Datamash example inputs usually don't have headers unless --header-in is used.
    // But tva ALWAYS expects headers.
    // So if I feed the raw datamash input "a\tx\t1" as line 1, tva will treat "a", "x", "1" as HEADERS.
    // I need to add a header line for tva tests.

    // Retrying Scenario 1 with header
    let mut file1_h = tempfile::NamedTempFile::new()?;
    writeln!(file1_h, "ID\tKey\tVal")?;
    writeln!(file1_h, "a\tx\t1")?;
    writeln!(file1_h, "a\ty\t2")?;
    writeln!(file1_h, "a\tx\t3")?;
    let path1_h = file1_h.path().to_str().unwrap();

    let mut cmd1 = cargo_bin_cmd!("tva");
    let output1 = cmd1
        .arg("wider")
        .arg(path1_h)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .output()?;

    // Expected: ID  x  y
    //           a   3  2
    // Note: Column order of x/y depends on appearance order if not sorted.
    // x appears first (line 2), y appears second (line 3).
    // So x then y.
    let expected1 = "ID\tx\ty\na\t3\t2\n";
    let stdout1 = String::from_utf8(output1.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout1, expected1);

    // Scenario 2: Missing values with custom filler (mirrors Datamash in3/out3_xx)
    // datamash: --filler XX crosstab 1,2 first 3
    let mut file2 = tempfile::NamedTempFile::new()?;
    writeln!(file2, "ID\tKey\tVal")?;
    writeln!(file2, "a\tx\t1")?;
    writeln!(file2, "a\ty\t2")?;
    writeln!(file2, "b\tx\t3")?;
    let path2 = file2.path().to_str().unwrap();

    let mut cmd2 = cargo_bin_cmd!("tva");
    let output2 = cmd2
        .arg("wider")
        .arg(path2)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--values-fill")
        .arg("XX")
        .output()?;

    // Expected: ID x y
    //           a  1 2
    //           b  3 XX
    let expected2 = "ID\tx\ty\na\t1\t2\nb\t3\tXX\n";
    let stdout2 = String::from_utf8(output2.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout2, expected2);

    Ok(())
}

#[test]
fn wider_aggregation_ops() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tname\tval")?;
    writeln!(file, "A\tX\t10")?;
    writeln!(file, "A\tX\t20")?; // Duplicate A-X, values 10 and 20
    writeln!(file, "B\tY\t5")?;
    writeln!(file, "B\tY\t15")?; // Duplicate B-Y, values 5 and 15
    writeln!(file, "C\tZ\t100")?;
    let path = file.path().to_str().unwrap();

    // 1. Test SUM
    let output_sum = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("sum")
        .output()?;

    // Expected:
    // ID X   Y   Z
    // A  30  -   -
    // B  -   20  -
    // C  -   -   100
    // (Actual formatting depends on column order, which is insertion order by default: X, Y, Z)
    // Empty cells are empty string by default.
    let expected_sum = "ID\tX\tY\tZ\nA\t30\t\t\nB\t\t20\t\nC\t\t\t100\n";
    let stdout_sum = String::from_utf8(output_sum.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout_sum, expected_sum);

    // 2. Test MEAN
    let mut cmd2 = cargo_bin_cmd!("tva");
    let output_mean = cmd2
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("mean")
        .output()?;
    
    // A: (10+20)/2 = 15
    // B: (5+15)/2 = 10
    // C: 100
    let expected_mean = "ID\tX\tY\tZ\nA\t15\t\t\nB\t\t10\t\nC\t\t\t100\n";
    let stdout_mean = String::from_utf8(output_mean.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout_mean, expected_mean);

    // 3. Test COUNT (crosstab)
    let mut cmd3 = cargo_bin_cmd!("tva");
    let output_count = cmd3
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        // No values-from needed for count
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("count")
        .output()?;

    // A: 2
    // B: 2
    // C: 1
    let expected_count = "ID\tX\tY\tZ\nA\t2\t\t\nB\t\t2\t\nC\t\t\t1\n";
    let stdout_count = String::from_utf8(output_count.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout_count, expected_count);

    Ok(())
}
