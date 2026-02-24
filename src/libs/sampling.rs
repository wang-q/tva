use rapidhash::RapidRng;

fn write_with_optional_random<W: std::io::Write>(
    writer: &mut W,
    row: &str,
    rng: &mut RapidRng,
    print_random: bool,
    random_value: Option<f64>,
) -> anyhow::Result<()> {
    if print_random {
        let v = match random_value {
            Some(x) => x,
            None => rng.next() as f64 / (u64::MAX as f64 + 1.0),
        };
        let value_str = format!("{:.10}", v);
        writer.write_all(value_str.as_bytes())?;
        writer.write_all(b"\t")?;
    }
    writer.write_all(row.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub fn bernoulli_sample<W: std::io::Write>(
    writer: &mut W,
    rows: &[String],
    prob: f64,
    rng: &mut RapidRng,
    print_random: bool,
) -> anyhow::Result<()> {
    for row in rows {
        if row.is_empty() {
            writer.write_all(b"\n")?;
            continue;
        }

        let r = rng.next() as f64 / (u64::MAX as f64 + 1.0);
        if r < prob {
            write_with_optional_random(writer, row, rng, print_random, Some(r))?;
        }
    }

    Ok(())
}

pub fn shuffle_rows<W: std::io::Write>(
    writer: &mut W,
    mut rows: Vec<String>,
    rng: &mut RapidRng,
    print_random: bool,
) -> anyhow::Result<()> {
    let len = rows.len();
    for i in (1..len).rev() {
        let j = (rng.next() as usize) % (i + 1);
        rows.swap(i, j);
    }

    for row in rows {
        write_with_optional_random(writer, &row, rng, print_random, None)?;
    }

    Ok(())
}

pub fn compat_random_sample<W: std::io::Write>(
    writer: &mut W,
    rows: &[String],
    k: usize,
    rng: &mut RapidRng,
    print_random: bool,
) -> anyhow::Result<()> {
    let n = rows.len();
    if n == 0 {
        return Ok(());
    }

    let sample_size = if k == 0 || k >= n { n } else { k };

    let mut keyed_indices: Vec<(f64, usize)> = Vec::with_capacity(n);
    for (idx, _) in rows.iter().enumerate() {
        let r = rng.next() as f64 / (u64::MAX as f64 + 1.0);
        keyed_indices.push((r, idx));
    }

    keyed_indices.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for (r, idx) in keyed_indices.into_iter().take(sample_size) {
        let row = &rows[idx];
        write_with_optional_random(writer, row, rng, print_random, Some(r))?;
    }

    Ok(())
}

pub fn fixed_size_sample<W: std::io::Write>(
    writer: &mut W,
    rows: Vec<String>,
    k: usize,
    rng: &mut RapidRng,
    print_random: bool,
) -> anyhow::Result<()> {
    let n = rows.len();

    if k == 0 || n == 0 {
        return Ok(());
    }

    if k >= n {
        return shuffle_rows(writer, rows, rng, print_random);
    }
    let mut sample: Vec<String> = Vec::with_capacity(k);

    for (i, row) in rows.into_iter().enumerate() {
        if i < k {
            sample.push(row);
        } else {
            let j = rng.next() as usize % (i + 1);
            if j < k {
                sample[j] = row;
            }
        }
    }

    shuffle_rows(writer, sample, rng, print_random)
}

pub fn sample_with_replacement<W: std::io::Write>(
    writer: &mut W,
    rows: &[String],
    k: usize,
    rng: &mut RapidRng,
) -> anyhow::Result<()> {
    if k == 0 || rows.is_empty() {
        return Ok(());
    }

    let n = rows.len();
    for _ in 0..k {
        let idx = (rng.next() as usize) % n;
        let row = &rows[idx];
        writer.write_all(row.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}

pub fn fixed_size_sample_inorder<W: std::io::Write>(
    writer: &mut W,
    rows: &[String],
    k: usize,
    rng: &mut RapidRng,
    print_random: bool,
) -> anyhow::Result<()> {
    let n = rows.len();

    if k == 0 || n == 0 {
        return Ok(());
    }

    if k >= n {
        for row in rows {
            write_with_optional_random(writer, row, rng, print_random, None)?;
        }
        return Ok(());
    }

    let mut indices: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rng.next() as usize) % (i + 1);
        indices.swap(i, j);
    }

    indices.truncate(k);
    indices.sort_unstable();

    for idx in indices {
        let row = &rows[idx];
        write_with_optional_random(writer, row, rng, print_random, None)?;
    }

    Ok(())
}

pub fn weighted_fixed_size_sample<W: std::io::Write>(
    writer: &mut W,
    rows: &[String],
    k: usize,
    has_header: bool,
    header_line: Option<&str>,
    weight_spec: &str,
    rng: &mut RapidRng,
    print_random: bool,
) -> anyhow::Result<()> {
    use crate::libs::fields::{parse_field_list_with_header, Header};

    let n = rows.len();

    if k == 0 || n == 0 {
        return Ok(());
    }

    let delimiter = '\t';
    let header = if has_header {
        header_line.map(|line| Header::from_line(line, delimiter))
    } else {
        None
    };

    let field_indices =
        parse_field_list_with_header(weight_spec, header.as_ref(), delimiter)
            .map_err(|e| anyhow::anyhow!("tva sample: {}", e))?;

    if field_indices.len() != 1 {
        return Err(anyhow::anyhow!(
            "tva sample: --weight-field/-w must select exactly one field"
        ));
    }

    let field_idx = field_indices[0];

    let mut weighted: Vec<(f64, &String)> = Vec::with_capacity(n);

    for row in rows {
        if row.is_empty() {
            continue;
        }
        let cols: Vec<&str> = row.split(delimiter).collect();
        if field_idx == 0 || field_idx > cols.len() {
            return Err(anyhow::anyhow!(
                "tva sample: weight field index {} out of range",
                field_idx
            ));
        }
        let w_str = cols[field_idx - 1].trim();
        if w_str.is_empty() {
            continue;
        }
        let w: f64 = w_str.parse().map_err(|_| {
            anyhow::anyhow!("tva sample: weight value `{}` is not a valid number", w_str)
        })?;
        if w <= 0.0 {
            continue;
        }
        let u = rng.next() as f64 / (u64::MAX as f64 + 1.0);
        let key = -u.ln() / w;
        weighted.push((key, row));
    }

    if weighted.is_empty() {
        return Ok(());
    }

    weighted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let limit = k.min(weighted.len());
    for (_, row) in weighted.into_iter().take(limit) {
        write_with_optional_random(writer, row, rng, print_random, None)?;
    }

    Ok(())
}

pub fn distinct_bernoulli_sample<W: std::io::Write>(
    writer: &mut W,
    rows: &[String],
    prob: f64,
    has_header: bool,
    header_line: Option<&str>,
    key_spec: &str,
    rng: &mut RapidRng,
    print_random: bool,
) -> anyhow::Result<()> {
    use crate::libs::fields::{parse_field_list_with_header, Header};
    use std::collections::HashMap;

    if prob <= 0.0 {
        return Ok(());
    }

    let delimiter = '\t';

    let header = if has_header {
        header_line.map(|line| Header::from_line(line, delimiter))
    } else {
        None
    };

    let spec_trimmed = key_spec.trim();
    let indices = if spec_trimmed == "0" {
        Vec::new()
    } else {
        parse_field_list_with_header(spec_trimmed, header.as_ref(), delimiter)
            .map_err(|e| anyhow::anyhow!("tva sample: {}", e))?
    };

    let mut decisions: HashMap<String, (bool, f64)> = HashMap::new();

    for row in rows {
        if row.is_empty() {
            writer.write_all(b"\n")?;
            continue;
        }

        let key = if spec_trimmed == "0" {
            row.clone()
        } else {
            let cols: Vec<&str> = row.split(delimiter).collect();
            if indices.is_empty() {
                return Err(anyhow::anyhow!(
                    "tva sample: --key-fields/-k must select at least one field"
                ));
            }
            let mut parts: Vec<&str> = Vec::with_capacity(indices.len());
            for idx in &indices {
                if *idx == 0 || *idx > cols.len() {
                    return Err(anyhow::anyhow!(
                        "tva sample: key field index {} out of range",
                        idx
                    ));
                }
                parts.push(cols[*idx - 1]);
            }
            parts.join("\x1f")
        };

        let (keep, rand_val) = if let Some(&(k, v)) = decisions.get(&key) {
            (k, v)
        } else {
            let r = rng.next() as f64 / (u64::MAX as f64 + 1.0);
            let k = r < prob;
            decisions.insert(key, (k, r));
            (k, r)
        };

        if keep {
            write_with_optional_random(writer, row, rng, print_random, Some(rand_val))?;
        }
    }

    Ok(())
}
