use crate::libs::sampling::*;
use crate::libs::tsv::reader::TsvReader;
use ahash::AHashMap;
use clap::*;
use rapidhash::RapidRng;
use std::io::Write;

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

struct SampleConfig {
    infiles: Vec<String>,
    has_header: bool,
    num_opt: u64,
    prob_opt: Option<f64>,
    weight_field: Option<String>,
    print_random: bool,
    gen_random_inorder: bool,
    random_value_header: String,
    compatibility_mode: bool,
    key_fields: Option<String>,
    inorder: bool,
    static_seed: bool,
    seed_value: u64,
    replace: bool,
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    execute_inner(args).map_err(|e| anyhow::anyhow!("tva sample: {}", e))
}

fn execute_inner(args: &ArgMatches) -> anyhow::Result<()> {
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

    let config = SampleConfig {
        infiles,
        has_header,
        num_opt,
        prob_opt,
        weight_field,
        print_random,
        gen_random_inorder,
        random_value_header,
        compatibility_mode,
        key_fields,
        inorder,
        static_seed,
        seed_value,
        replace,
    };

    let outfile = args.get_one::<String>("outfile").unwrap();
    if outfile == "stdout" {
        let stdout = std::io::stdout();
        let handle = stdout.lock();
        let mut writer = std::io::BufWriter::with_capacity(128 * 1024, handle);
        run_sampling(config, &mut writer)
    } else {
        let file = std::fs::File::create(outfile)?;
        let mut writer = std::io::BufWriter::with_capacity(128 * 1024, file);
        run_sampling(config, &mut writer)
    }
}

enum SamplerEnum {
    Bernoulli(BernoulliSampler),
    Distinct(DistinctBernoulliSampler),
    Reservoir(ReservoirSampler),
    Weighted(WeightedReservoirSampler),
    Shuffle(ShuffleSampler),
    Inorder(InorderSampler),
    Replacement(ReplacementSampler),
    Compat(CompatRandomSampler),
}

impl Sampler for SamplerEnum {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        match self {
            SamplerEnum::Bernoulli(s) => s.process(record, writer, rng),
            SamplerEnum::Distinct(s) => s.process(record, writer, rng),
            SamplerEnum::Reservoir(s) => s.process(record, writer, rng),
            SamplerEnum::Weighted(s) => s.process(record, writer, rng),
            SamplerEnum::Shuffle(s) => s.process(record, writer, rng),
            SamplerEnum::Inorder(s) => s.process(record, writer, rng),
            SamplerEnum::Replacement(s) => s.process(record, writer, rng),
            SamplerEnum::Compat(s) => s.process(record, writer, rng),
        }
    }
    fn finalize<W2: Write>(
        &mut self,
        writer: &mut W2,
        rng: &mut RapidRng,
        print_random: bool,
    ) -> anyhow::Result<()> {
        match self {
            SamplerEnum::Bernoulli(s) => s.finalize(writer, rng, print_random),
            SamplerEnum::Distinct(s) => s.finalize(writer, rng, print_random),
            SamplerEnum::Reservoir(s) => s.finalize(writer, rng, print_random),
            SamplerEnum::Weighted(s) => s.finalize(writer, rng, print_random),
            SamplerEnum::Shuffle(s) => s.finalize(writer, rng, print_random),
            SamplerEnum::Inorder(s) => s.finalize(writer, rng, print_random),
            SamplerEnum::Replacement(s) => s.finalize(writer, rng, print_random),
            SamplerEnum::Compat(s) => s.finalize(writer, rng, print_random),
        }
    }
}

fn run_sampling<W: Write>(config: SampleConfig, writer: &mut W) -> anyhow::Result<()> {
    if config.gen_random_inorder {
        run_gen_random_inorder(config, writer)
    } else {
        run_standard_sampling(config, writer)
    }
}

fn run_gen_random_inorder<W: Write>(
    config: SampleConfig,
    writer: &mut W,
) -> anyhow::Result<()> {
    let mut rng = if !config.static_seed && config.seed_value == 0 {
        RapidRng::default()
    } else if config.seed_value != 0 {
        RapidRng::new(config.seed_value)
    } else {
        RapidRng::new(2438424139)
    };

    let mut header_line: Option<Vec<u8>> = None;
    let mut header_written = false;

    for input in crate::libs::io::raw_input_sources(&config.infiles) {
        let mut reader = TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut is_first_record = true;

        if !header_written && header_line.is_some() {
            let header = header_line.as_ref().unwrap();
            writer.write_all(config.random_value_header.as_bytes())?;
            writer.write_all(b"\t")?;
            writer.write_all(header)?;
            writer.write_all(b"\n")?;
            header_written = true;
        }

        reader.for_each_record(|record| {
            if record.is_empty() {
                writer.write_all(b"\n")?;
                return Ok(());
            }

            if config.has_header && is_first_record {
                is_first_record = false;
                if header_line.is_none() {
                    header_line = Some(record.to_vec());
                    // Write header immediately
                    writer.write_all(config.random_value_header.as_bytes())?;
                    writer.write_all(b"\t")?;
                    writer.write_all(record)?;
                    writer.write_all(b"\n")?;
                    header_written = true;
                }
                return Ok(());
            }

            // Not header or no header mode
            let r = rng.next() as f64 * crate::libs::sampling::INV_U64_MAX_PLUS_1;
            let mut buffer = ryu::Buffer::new();
            let printed = buffer.format(r);
            writer.write_all(printed.as_bytes())?;
            writer.write_all(b"\t")?;
            writer.write_all(record)?;
            writer.write_all(b"\n")?;
            Ok(())
        })?;
    }

    Ok(())
}

fn run_standard_sampling<W: Write>(
    config: SampleConfig,
    writer: &mut W,
) -> anyhow::Result<()> {
    let SampleConfig {
        infiles,
        has_header,
        num_opt,
        prob_opt,
        weight_field,
        print_random,
        random_value_header,
        compatibility_mode,
        key_fields,
        inorder,
        static_seed,
        seed_value,
        replace,
        ..
    } = config;

    let mut rng = if !static_seed && seed_value == 0 {
        RapidRng::default()
    } else if seed_value != 0 {
        RapidRng::new(seed_value)
    } else {
        RapidRng::new(2438424139)
    };

    let mut sampler: SamplerEnum = if let Some(p) = prob_opt {
        if let Some(ref _key_spec) = key_fields {
            SamplerEnum::Distinct(DistinctBernoulliSampler {
                prob: p,
                key_field_indices: Vec::new(), // To be filled
                print_random,
                decisions: AHashMap::new(),
                key_buffer: Vec::new(),
            })
        } else {
            // Generate initial skip for Bernoulli sampler
            let initial_skip = if p >= 1.0 {
                0
            } else {
                let u = rng.next() as f64 * crate::libs::sampling::INV_U64_MAX_PLUS_1;
                let u = if u < 1e-10 { 1e-10 } else { u };
                (u.ln() / (1.0 - p).ln()).floor() as usize
            };
            SamplerEnum::Bernoulli(BernoulliSampler {
                prob: p,
                print_random,
                skip_counter: initial_skip,
            })
        }
    } else if replace && num_opt > 0 {
        SamplerEnum::Replacement(ReplacementSampler {
            k: num_opt as usize,
            rows: Vec::new(),
        })
    } else if let Some(ref _weight_spec) = weight_field {
        SamplerEnum::Weighted(WeightedReservoirSampler {
            k: num_opt as usize,
            weight_field_idx: 0, // To be filled
            heap: std::collections::BinaryHeap::new(),
        })
    } else if num_opt == 0 {
        if compatibility_mode {
            SamplerEnum::Compat(CompatRandomSampler {
                k: 0,
                rows: Vec::new(),
            })
        } else {
            SamplerEnum::Shuffle(ShuffleSampler { rows: Vec::new() })
        }
    } else if inorder {
        SamplerEnum::Inorder(InorderSampler {
            k: num_opt as usize,
            rows: Vec::new(),
        })
    } else if compatibility_mode {
        SamplerEnum::Compat(CompatRandomSampler {
            k: num_opt as usize,
            rows: Vec::new(),
        })
    } else {
        SamplerEnum::Reservoir(ReservoirSampler {
            k: num_opt as usize,
            reservoir: Vec::new(),
            count: 0,
        })
    };

    let mut header_line: Option<Vec<u8>> = None;
    let mut header_written = false;
    let mut sampler_initialized = false;

    let distinct_key_spec = key_fields.as_deref();
    let weighted_weight_spec = weight_field.as_deref();

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader = TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut is_first_record = true;

        reader.for_each_record(|record| {
            if record.is_empty() {
                writer.write_all(b"\n")?;
                return Ok(());
            }
            if has_header && is_first_record {
                is_first_record = false;
                if header_line.is_none() {
                    header_line = Some(record.to_vec());

                    // Init sampler config if needed
                    if !sampler_initialized {
                        match &mut sampler {
                            SamplerEnum::Distinct(s) => {
                                use crate::libs::tsv::fields::{
                                    parse_field_list_with_header, Header,
                                };
                                let record_str =
                                    std::str::from_utf8(record).map_err(|e| {
                                        std::io::Error::new(
                                            std::io::ErrorKind::InvalidData,
                                            format!("{}", e),
                                        )
                                    })?;
                                let header = Header::from_line(record_str, '\t');
                                let spec = distinct_key_spec.unwrap();
                                let indices =
                                    if spec == "0" {
                                        Vec::new()
                                    } else {
                                        parse_field_list_with_header(
                                            spec,
                                            Some(&header),
                                            '\t',
                                        )
                                        .map_err(|e| {
                                            std::io::Error::new(
                                                std::io::ErrorKind::InvalidInput,
                                                format!("{}", e),
                                            )
                                        })?
                                    };
                                s.key_field_indices = indices;
                            }
                            SamplerEnum::Weighted(s) => {
                                use crate::libs::tsv::fields::{
                                    parse_field_list_with_header, Header,
                                };
                                let record_str =
                                    std::str::from_utf8(record).map_err(|e| {
                                        std::io::Error::new(
                                            std::io::ErrorKind::InvalidData,
                                            format!("{}", e),
                                        )
                                    })?;
                                let header = Header::from_line(record_str, '\t');
                                let spec = weighted_weight_spec.unwrap();
                                let indices = parse_field_list_with_header(
                                    spec,
                                    Some(&header),
                                    '\t',
                                )
                                .map_err(|e| {
                                    std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!("{}", e),
                                    )
                                })?;
                                if indices.len() != 1 {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        "--weight-field must select exactly one field",
                                    )
                                    .into());
                                }
                                s.weight_field_idx = indices[0];
                            }
                            _ => {}
                        }
                        sampler_initialized = true;
                    }

                    // Write header if streaming immediately (Bernoulli/Distinct)
                    if let SamplerEnum::Bernoulli(_) | SamplerEnum::Distinct(_) = sampler {
                        if print_random {
                            writer.write_all(random_value_header.as_bytes())?;
                            writer.write_all(b"\t")?;
                        }
                        writer.write_all(record)?;
                        writer.write_all(b"\n")?;
                        header_written = true;
                    }
                }
                // Skip header for subsequent files
                return Ok(());
            }

            // Not header or no header mode
            // Init sampler if no header and first record
            if !sampler_initialized {
                match &mut sampler {
                    SamplerEnum::Distinct(s) => {
                        use crate::libs::tsv::fields::parse_field_list_with_header;
                        let spec = distinct_key_spec.unwrap();
                        let indices = if spec == "0" {
                            Vec::new()
                        } else {
                            parse_field_list_with_header(spec, None, '\t').map_err(
                                |e| {
                                    std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!("{}", e),
                                    )
                                },
                            )?
                        };
                        s.key_field_indices = indices;
                    }
                    SamplerEnum::Weighted(s) => {
                        use crate::libs::tsv::fields::parse_field_list_with_header;
                        let spec = weighted_weight_spec.unwrap();
                        let indices = parse_field_list_with_header(spec, None, '\t')
                            .map_err(|e| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!("{}", e),
                                )
                            })?;
                        if indices.len() != 1 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "--weight-field must select exactly one field",
                            )
                            .into());
                        }
                        s.weight_field_idx = indices[0];
                    }
                    _ => {}
                }
                sampler_initialized = true;
            }

            sampler
                .process(record, writer, &mut rng)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            Ok(())
        })?;
    }

    // Write header for non-streaming samplers
    if !header_written {
        if let Some(header) = &header_line {
            if print_random {
                writer.write_all(random_value_header.as_bytes())?;
                writer.write_all(b"\t")?;
            }
            writer.write_all(header)?;
            writer.write_all(b"\n")?;
        }
    }

    sampler.finalize(writer, &mut rng, print_random)?;

    Ok(())
}
