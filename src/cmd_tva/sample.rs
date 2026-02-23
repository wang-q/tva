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

Notes:
- This implementation currently supports only unweighted sampling and does not
  provide distinct sampling or random value printing.
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

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let has_header = args.get_flag("header");
    let num_opt = args.get_one::<u64>("num").cloned().unwrap_or(0);
    let prob_opt = args.get_one::<f64>("prob").cloned();
    let inorder = args.get_flag("inorder");
    let static_seed = args.get_flag("static-seed");
    let seed_value = args.get_one::<u64>("seed-value").cloned().unwrap_or(0);
    let replace = args.get_flag("replace");

    if num_opt > 0 && prob_opt.is_some() {
        eprintln!("tva sample: --num/-n and --prob/-p cannot be used together");
        std::process::exit(1);
    }

    if replace && prob_opt.is_some() {
        eprintln!("tva sample: --replace/-r cannot be used with --prob/-p");
        std::process::exit(1);
    }

    if replace && num_opt == 0 {
        eprintln!("tva sample: --replace/-r requires --num/-n greater than 0");
        std::process::exit(1);
    }

    if inorder && (prob_opt.is_some() || replace || num_opt == 0) {
        eprintln!(
            "tva sample: --inorder/-i requires --num/-n without --replace/-r or --prob/-p"
        );
        std::process::exit(1);
    }

    if let Some(p) = prob_opt {
        if !(p > 0.0 && p <= 1.0) {
            eprintln!(
                "tva sample: invalid --prob/-p value {} (must satisfy 0.0 < prob <= 1.0)",
                p
            );
            std::process::exit(1);
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

    if let Some(header) = &header_line {
        writer.write_all(header.as_bytes())?;
        writer.write_all(b"\n")?;
        header_written = true;
    }

    if let Some(p) = prob_opt {
        bernoulli_sample(&mut writer, data_rows, p, &mut rng)?;
        return Ok(());
    }

    if replace && num_opt > 0 {
        sample_with_replacement(
            &mut writer,
            &data_rows,
            num_opt as usize,
            &mut rng,
        )?;
    } else if num_opt == 0 {
        shuffle_rows(&mut writer, data_rows, &mut rng)?;
    } else if inorder {
        fixed_size_sample_inorder(&mut writer, &data_rows, num_opt as usize, &mut rng)?;
    } else {
        fixed_size_sample(&mut writer, data_rows, num_opt as usize, &mut rng)?;
    }

    if !header_written && header_line.is_none() {
        // nothing special to do
    }

    Ok(())
}

fn bernoulli_sample(
    writer: &mut Box<dyn std::io::Write>,
    rows: Vec<String>,
    prob: f64,
    rng: &mut RapidRng,
) -> anyhow::Result<()> {
    for row in rows {
        if row.is_empty() {
            writer.write_all(b"\n")?;
            continue;
        }

        let r = rng.next() as f64 / (u64::MAX as f64 + 1.0);
        if r < prob {
            writer.write_all(row.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    Ok(())
}

fn shuffle_rows(
    writer: &mut Box<dyn std::io::Write>,
    mut rows: Vec<String>,
    rng: &mut RapidRng,
) -> anyhow::Result<()> {
    let len = rows.len();
    for i in (1..len).rev() {
        let j = (rng.next() as usize) % (i + 1);
        rows.swap(i, j);
    }

    for row in rows {
        writer.write_all(row.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}

fn fixed_size_sample(
    writer: &mut Box<dyn std::io::Write>,
    rows: Vec<String>,
    k: usize,
    rng: &mut RapidRng,
) -> anyhow::Result<()> {
    let n = rows.len();

    if k == 0 || n == 0 {
        return Ok(());
    }

    if k >= n {
        return shuffle_rows(writer, rows, rng);
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

    shuffle_rows(writer, sample, rng)
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
) -> anyhow::Result<()> {
    let n = rows.len();

    if k == 0 || n == 0 {
        return Ok(());
    }

    if k >= n {
        for row in rows {
            writer.write_all(row.as_bytes())?;
            writer.write_all(b"\n")?;
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
        writer.write_all(row.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}
