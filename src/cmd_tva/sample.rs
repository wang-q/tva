use crate::libs::sampling::*;
use clap::*;
use rapidhash::RapidRng;
use std::io::BufRead;

pub fn make_subcommand() -> Command {
    Command::new("sample")
        .about("Samples or shuffles TSV rows")
        .after_help(include_str!("../../docs/help/sample.md"))
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
    let key_fields = args.get_one::<String>("key-fields").map(|s| s.to_string());
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

            data_rows.push(line);
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
            let config = DistinctSampleConfig {
                prob: p,
                has_header,
                header_line: header_line.as_deref(),
                key_spec,
                print_random,
            };
            distinct_bernoulli_sample(&mut writer, &data_rows, &mut rng, config)?;
        } else {
            bernoulli_sample(&mut writer, &data_rows, p, &mut rng, print_random)?;
        }
        return Ok(());
    }

    if replace && num_opt > 0 {
        sample_with_replacement(&mut writer, &data_rows, num_opt as usize, &mut rng)?;
    } else if let Some(weight_spec) = weight_field {
        let config = WeightedSampleConfig {
            k: num_opt as usize,
            has_header,
            header_line: header_line.as_deref(),
            weight_spec: &weight_spec,
            print_random,
        };
        weighted_fixed_size_sample(&mut writer, &data_rows, &mut rng, config)?;
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
        fixed_size_sample(
            &mut writer,
            data_rows,
            num_opt as usize,
            &mut rng,
            print_random,
        )?;
    }

    if !header_written && header_line.is_none() {
        // nothing special to do
    }

    Ok(())
}
