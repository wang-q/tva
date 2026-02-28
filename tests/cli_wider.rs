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
    use std::io::Write;
    let mut file1 = tempfile::NamedTempFile::new()?;
    writeln!(file1, "ID\tKey\tVal")?;
    writeln!(file1, "a\tx\t1")?;
    writeln!(file1, "a\ty\t2")?;
    writeln!(file1, "a\tx\t3")?;
    let path1 = file1.path().to_str().unwrap();

    let output1 = cmd
        .arg("wider")
        .arg(path1)
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

#[test]
fn wider_extended_stats() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tKey\tVal")?;
    // Group A: 1, 3, 5.
    // Min=1, Max=5, Median=3, Mean=3
    // Range=4, Stdev=2, Variance=4
    writeln!(file, "A\tX\t1")?;
    writeln!(file, "A\tX\t3")?;
    writeln!(file, "A\tX\t5")?;
    
    // Group B: 2, 2, 8.
    // Min=2, Max=8, Median=2, Mode=2
    writeln!(file, "B\tX\t2")?;
    writeln!(file, "B\tX\t2")?;
    writeln!(file, "B\tX\t8")?;
    let path = file.path().to_str().unwrap();

    // 1. Min/Max/Range
    let output_min = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("min")
        .output()?;
    let stdout_min = String::from_utf8(output_min.stdout)?.replace("\r\n", "\n");
    assert!(stdout_min.contains("A\t1"));
    assert!(stdout_min.contains("B\t2"));

    let mut cmd2 = cargo_bin_cmd!("tva");
    let output_max = cmd2
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("max")
        .output()?;
    let stdout_max = String::from_utf8(output_max.stdout)?.replace("\r\n", "\n");
    assert!(stdout_max.contains("A\t5"));
    assert!(stdout_max.contains("B\t8"));

    // 2. Median
    let mut cmd3 = cargo_bin_cmd!("tva");
    let output_median = cmd3
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("median")
        .output()?;
    let stdout_median = String::from_utf8(output_median.stdout)?.replace("\r\n", "\n");
    assert!(stdout_median.contains("A\t3"));
    assert!(stdout_median.contains("B\t2"));

    // 3. Mode (B has mode 2)
    let mut cmd4 = cargo_bin_cmd!("tva");
    let output_mode = cmd4
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("mode")
        .output()?;
    let stdout_mode = String::from_utf8(output_mode.stdout)?.replace("\r\n", "\n");
    // Mode for A is 1 (or 3 or 5, implementation dependent for ties, usually first/lowest?)
    // Our implementation sorts by count desc, then value asc. So 1.
    assert!(stdout_mode.contains("A\t1")); 
    assert!(stdout_mode.contains("B\t2"));

    Ok(())
}

#[test]
fn wider_first_last() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tKey\tVal")?;
    writeln!(file, "A\tX\tfirst_val")?;
    writeln!(file, "A\tX\tmiddle_val")?;
    writeln!(file, "A\tX\tlast_val")?;
    let path = file.path().to_str().unwrap();

    // First
    let output_first = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("first")
        .output()?;
    let stdout_first = String::from_utf8(output_first.stdout)?.replace("\r\n", "\n");
    assert!(stdout_first.contains("A\tfirst_val"));

    // Last
    let mut cmd2 = cargo_bin_cmd!("tva");
    let output_last = cmd2
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("last")
        .output()?;
    let stdout_last = String::from_utf8(output_last.stdout)?.replace("\r\n", "\n");
    assert!(stdout_last.contains("A\tlast_val"));

    Ok(())
}

#[test]
fn wider_quartiles_iqr() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tKey\tVal")?;
    // Data: 1, 2, 3, 4, 5
    // Q1 (25th): 2
    // Q3 (75th): 4
    // IQR: 4 - 2 = 2
    // Median (50th): 3
    for i in 1..=5 {
        writeln!(file, "A\tX\t{}", i)?;
    }
    let path = file.path().to_str().unwrap();

    // Q1
    let output_q1 = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("q1")
        .output()?;
    let stdout_q1 = String::from_utf8(output_q1.stdout)?.replace("\r\n", "\n");
    assert!(stdout_q1.contains("A\t2"));

    // Q3
    let mut cmd2 = cargo_bin_cmd!("tva");
    let output_q3 = cmd2
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("q3")
        .output()?;
    let stdout_q3 = String::from_utf8(output_q3.stdout)?.replace("\r\n", "\n");
    assert!(stdout_q3.contains("A\t4"));

    // IQR
    let mut cmd3 = cargo_bin_cmd!("tva");
    let output_iqr = cmd3
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("iqr")
        .output()?;
    let stdout_iqr = String::from_utf8(output_iqr.stdout)?.replace("\r\n", "\n");
    assert!(stdout_iqr.contains("A\t2"));

    Ok(())
}

#[test]
fn wider_advanced_math_stats() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tKey\tVal")?;
    // Data: 2, 8
    // Mean = 5
    // GeoMean = sqrt(2*8) = 4
    // HarmMean = 2 / (1/2 + 1/8) = 2 / (0.625) = 3.2
    // Variance (sample): ((2-5)^2 + (8-5)^2) / (2-1) = (9 + 9)/1 = 18
    // Stdev: sqrt(18) ≈ 4.2426
    // CV: Stdev / Mean = 4.2426 / 5 ≈ 0.8485
    writeln!(file, "A\tX\t2")?;
    writeln!(file, "A\tX\t8")?;
    let path = file.path().to_str().unwrap();

    // GeoMean
    let output_geo = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("geomean")
        .output()?;
    let stdout_geo = String::from_utf8(output_geo.stdout)?.replace("\r\n", "\n");
    assert!(stdout_geo.contains("A\t4"));

    // HarmMean
    let mut cmd2 = cargo_bin_cmd!("tva");
    let output_harm = cmd2
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("harmmean")
        .output()?;
    let stdout_harm = String::from_utf8(output_harm.stdout)?.replace("\r\n", "\n");
    assert!(stdout_harm.contains("A\t3.2"));

    // Variance
    let mut cmd3 = cargo_bin_cmd!("tva");
    let output_var = cmd3
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("variance")
        .output()?;
    let stdout_var = String::from_utf8(output_var.stdout)?.replace("\r\n", "\n");
    assert!(stdout_var.contains("A\t18"));

    // Stdev
    let mut cmd4 = cargo_bin_cmd!("tva");
    let output_std = cmd4
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("stdev")
        .output()?;
    let stdout_std = String::from_utf8(output_std.stdout)?.replace("\r\n", "\n");
    // Check prefix
    assert!(stdout_std.contains("A\t4.242"));

    // CV
    let mut cmd5 = cargo_bin_cmd!("tva");
    let output_cv = cmd5
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("Key")
        .arg("--values-from")
        .arg("Val")
        .arg("--id-cols")
        .arg("ID")
        .arg("--op")
        .arg("cv")
        .output()?;
    let stdout_cv = String::from_utf8(output_cv.stdout)?.replace("\r\n", "\n");
    assert!(stdout_cv.contains("A\t0.848"));

    Ok(())
}
