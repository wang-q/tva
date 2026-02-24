use clap::*;
use rapidhash::RapidRng;
use std::io::BufRead;

pub fn make_subcommand() -> Command {
    Command::new("sample")
        .about("Samples or shuffles TSV rows")
        .after_help(
            r###"
Samples or shuffles tab-separated values (TSV) rows using simple random algorithms.

Modes:
- Default shuffle: With no sampling options, all input data rows are read and
  written in random order.
- Fixed-size sampling (--num/-n): Selects a random sample of N data rows and
  writes them in random order.
- Bernoulli sampling (--prob/-p): For each data row, independently includes the
  row in the output with probability PROB (0.0 < PROB <= 1.0). Row order is
  preserved.

Header behavior:
- --header / -H
  Treats the first non-empty line of the input as a header. The header is always
  written once at the top of the output. Sampling and shuffling are applied only
  to the remaining data rows.

Input:
- If no input files are given, or an input file is 'stdin', data is read
  from standard input.
- Files ending in '.gz' are transparently decompressed.

Output:
- By default, output is written to standard output.
- Use --outfile to write to a file instead.

Random value printing:
- Use --print-random to prepend a random value column to sampled rows.
- Use --gen-random-inorder to generate random values for all rows without
  changing input order.
"###,
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to sample from (default: stdin)"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first non-empty line as header and always keep it"),
        )
        .arg(
            Arg::new("num")
                .long("num")
                .short('n')
                .num_args(1)
                .value_parser(value_parser!(u64))
                .help("Maximum number of data rows to output (0 means all rows)"),
        )
        .arg(
            Arg::new("prob")
                .long("prob")
                .short('p')
                .num_args(1)
                .value_parser(value_parser!(f64))
                .help("Inclusion probability for Bernoulli sampling (0.0 < PROB <= 1.0)"),
        )
        .arg(
            Arg::new("weight-field")
                .long("weight-field")
                .short('w')
                .num_args(1)
                .help("Field (index or name) containing positive weights for rows"),
        )
        .arg(
            Arg::new("print-random")
                .long("print-random")
                .action(ArgAction::SetTrue)
                .help("Prepend a random value column to each sampled data row"),
        )
        .arg(
            Arg::new("gen-random-inorder")
                .long("gen-random-inorder")
                .action(ArgAction::SetTrue)
                .help("Generate random values for all rows without changing input order"),
        )
        .arg(
            Arg::new("random-value-header")
                .long("random-value-header")
                .num_args(1)
                .default_value("random_value")
                .help("Header to use for the random value column"),
        )
        .arg(
            Arg::new("compatibility-mode")
                .long("compatibility-mode")
                .action(ArgAction::SetTrue)
                .help("Use per-row random values so larger samples are supersets of smaller ones"),
        )
        .arg(
            Arg::new("key-fields")
                .long("key-fields")
                .short('k')
                .num_args(1)
                .help("Fields used as keys for distinct Bernoulli sampling (requires --prob)"),
        )
        .arg(
            Arg::new("inorder")
                .long("inorder")
                .short('i')
                .action(ArgAction::SetTrue)
                .help("For fixed-size sampling, preserve original input order"),
        )
        .arg(
            Arg::new("replace")
                .long("replace")
                .short('r')
                .action(ArgAction::SetTrue)
                .help("Sample with replacement; rows may be selected multiple times"),
        )
        .arg(
            Arg::new("static-seed")
                .long("static-seed")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("Use a fixed random seed so repeated runs produce the same result"),
        )
        .arg(
            Arg::new("seed-value")
                .long("seed-value")
                .short('v')
                .num_args(1)
                .value_parser(value_parser!(u64))
                .help("Explicit 64-bit seed value (non-zero) for the random generator"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

fn arg_error(msg: &str) -> ! {
    eprintln!("tva sample: {}", msg);
    std::process::exit(1);
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let has_header = args.get_flag("header");
    let num_opt = args.get_one::<u64>("num").cloned().unwrap_or(0);
    let prob_opt = args.get_one::<f64>("prob").cloned();
    let weight_field = args
        .get_one::<String>("weight-field")
        .map(|s| s.to_string());
    let print_random = args.get_flag("print-random");
    let gen_random_inorder = args.get_flag("gen-random-inorder");
    let random_value_header = args
        .get_one::<String>("random-value-header")
        .cloned()
        .unwrap_or_else(|| "random_value".to_string());
    let compatibility_mode = args.get_flag("compatibility-mode");
    let key_fields = args
        .get_one::<String>("key-fields")
        .map(|s| s.to_string());
    let inorder = args.get_flag("inorder");
    let static_seed = args.get_flag("static-seed");
    let seed_value = args.get_one::<u64>("seed-value").cloned().unwrap_or(0);
    let replace = args.get_flag("replace");

    if num_opt > 0 && prob_opt.is_some() && key_fields.is_none() {
        arg_error("--num/-n and --prob/-p cannot be used together");
    }

    if replace && prob_opt.is_some() {
        arg_error("--replace/-r cannot be used with --prob/-p");
    }

    if replace && num_opt == 0 {
        arg_error("--replace/-r requires --num/-n greater than 0");
    }

    if inorder && (prob_opt.is_some() || replace || num_opt == 0) {
        arg_error("--inorder/-i requires --num/-n without --replace/-r or --prob/-p");
    }

    if weight_field.is_some() && prob_opt.is_some() {
        arg_error("--weight-field/-w cannot be used with --prob/-p");
    }

    if weight_field.is_some() && replace {
        arg_error("--weight-field/-w cannot be used with --replace/-r");
    }

    if key_fields.is_some() && prob_opt.is_none() {
        arg_error("--key-fields/-k requires --prob/-p");
    }

    if key_fields.is_some()
        && (num_opt > 0 || replace || weight_field.is_some() || inorder)
    {
        arg_error(
            "--key-fields/-k cannot be used with --num/-n, --replace/-r, --inorder/-i, or --weight-field/-w",
        );
    }

    if print_random && gen_random_inorder {
        arg_error("--print-random cannot be used with --gen-random-inorder");
    }

    if gen_random_inorder
        && (prob_opt.is_some()
            || num_opt > 0
            || replace
            || weight_field.is_some()
            || key_fields.is_some()
            || inorder)
    {
        arg_error("--gen-random-inorder cannot be combined with sampling options");
    }

    if print_random && replace {
        arg_error("--print-random is not supported with --replace/-r");
    }

    if let Some(p) = prob_opt {
        if !(p > 0.0 && p <= 1.0) {
            arg_error(&format!(
                "invalid --prob/-p value {} (must satisfy 0.0 < prob <= 1.0)",
                p
            ));
        }
    }

    let mut rng = if !static_seed && seed_value == 0 {
        RapidRng::default()
    } else if seed_value != 0 {
        RapidRng::new(seed_value)
    } else {
        RapidRng::new(2438424139)
    };

    let mut data_rows: Vec<String> = Vec::new();
    let mut header_line: Option<String> = None;
    let mut header_written = false;

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;
        let mut is_first_nonempty = true;

        for line in reader.lines().map_while(Result::ok) {
            let mut line = line;
            if let Some('\r') = line.chars().last() {
                line.pop();
            }

            if has_header && is_first_nonempty && !line.is_empty() {
                if header_line.is_none() {
                    header_line = Some(line.clone());
                }
                is_first_nonempty = false;
                continue;
            }

            is_first_nonempty = false;

            if prob_opt.is_some() {
                data_rows.push(line);
            } else {
                data_rows.push(line);
            }
        }
    }

    if gen_random_inorder {
        if let Some(header) = &header_line {
            writer.write_all(random_value_header.as_bytes())?;
            writer.write_all(b"\t")?;
            writer.write_all(header.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        for row in &data_rows {
            if row.is_empty() {
                writer.write_all(b"\n")?;
                continue;
            }
            let r = rng.next() as f64 / (u64::MAX as f64 + 1.0);
            let value_str = format!("{:.10}", r);
            writer.write_all(value_str.as_bytes())?;
            writer.write_all(b"\t")?;
            writer.write_all(row.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        return Ok(());
    }

    if let Some(header) = &header_line {
        if print_random {
            writer.write_all(random_value_header.as_bytes())?;
            writer.write_all(b"\t")?;
        }
        writer.write_all(header.as_bytes())?;
        writer.write_all(b"\n")?;
        header_written = true;
    }

    if let Some(p) = prob_opt {
        if let Some(ref key_spec) = key_fields {
            distinct_bernoulli_sample(
                &mut writer,
                &data_rows,
                p,
                has_header,
                header_line.as_deref(),
                key_spec,
                &mut rng,
                print_random,
            )?;
        } else {
            bernoulli_sample(&mut writer, &data_rows, p, &mut rng, print_random)?;
        }
        return Ok(());
    }

    if replace && num_opt > 0 {
        sample_with_replacement(
            &mut writer,
            &data_rows,
            num_opt as usize,
            &mut rng,
        )?;
    } else if let Some(weight_spec) = weight_field {
        weighted_fixed_size_sample(
            &mut writer,
            &data_rows,
            num_opt as usize,
            has_header,
            header_line.as_deref(),
            &weight_spec,
            &mut rng,
            print_random,
        )?;
    } else if num_opt == 0 {
        if compatibility_mode {
            compat_random_sample(&mut writer, &data_rows, 0, &mut rng, print_random)?;
        } else {
            shuffle_rows(&mut writer, data_rows, &mut rng, print_random)?;
        }
    } else if inorder {
        fixed_size_sample_inorder(
            &mut writer,
            &data_rows,
            num_opt as usize,
            &mut rng,
            print_random,
        )?;
    } else if compatibility_mode {
        compat_random_sample(
            &mut writer,
            &data_rows,
            num_opt as usize,
            &mut rng,
            print_random,
        )?;
    } else {
        fixed_size_sample(&mut writer, data_rows, num_opt as usize, &mut rng, print_random)?;
    }

    if !header_written && header_line.is_none() {
        // nothing special to do
    }

    Ok(())
}

fn bernoulli_sample(
    writer: &mut Box<dyn std::io::Write>,
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

fn shuffle_rows(
    writer: &mut Box<dyn std::io::Write>,
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

fn compat_random_sample(
    writer: &mut Box<dyn std::io::Write>,
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

fn fixed_size_sample(
    writer: &mut Box<dyn std::io::Write>,
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

fn sample_with_replacement(
    writer: &mut Box<dyn std::io::Write>,
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

fn fixed_size_sample_inorder(
    writer: &mut Box<dyn std::io::Write>,
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

fn weighted_fixed_size_sample(
    writer: &mut Box<dyn std::io::Write>,
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
            anyhow::anyhow!(
                "tva sample: weight value `{}` is not a valid number",
                w_str
            )
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

fn distinct_bernoulli_sample(
    writer: &mut Box<dyn std::io::Write>,
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

fn write_with_optional_random(
    writer: &mut Box<dyn std::io::Write>,
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
