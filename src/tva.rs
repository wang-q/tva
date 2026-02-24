extern crate clap;

use clap::*;

use tva::cmd_tva;

fn main() -> anyhow::Result<()> {
    let app = Command::new("tva")
        .version(crate_version!())
        .author(crate_authors!())
        .about("tva: Tab-separated Values Assistant")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .arg(
            Arg::new("help-fields")
                .long("help-fields")
                .action(ArgAction::SetTrue)
                .help("Print help on field syntax and exit"),
        )
        .subcommand(cmd_tva::md::make_subcommand())
        .subcommand(cmd_tva::append::make_subcommand())
        .subcommand(cmd_tva::join::make_subcommand())
        .subcommand(cmd_tva::uniq::make_subcommand())
        .subcommand(cmd_tva::nl::make_subcommand())
        .subcommand(cmd_tva::keep_header::make_subcommand())
        .subcommand(cmd_tva::longer::make_subcommand())
        .subcommand(cmd_tva::check::make_subcommand())
        .subcommand(cmd_tva::transpose::make_subcommand())
        .subcommand(cmd_tva::sort::make_subcommand())
        .subcommand(cmd_tva::from_csv::make_subcommand())
        .subcommand(cmd_tva::select::make_subcommand())
        .subcommand(cmd_tva::sample::make_subcommand())
        .subcommand(cmd_tva::split::make_subcommand())
        .subcommand(cmd_tva::filter::make_subcommand())
        .subcommand(cmd_tva::stats::make_subcommand())
        .after_help(
            r###"
Tab-separated Values Assistant (tva): small toolbox for working with TSV files.

Currently implemented subcommands:
* Generic TSV: md, append, join, uniq, nl, transpose, sort, split, longer
* Table plumbing: keep-header, check
* Ingestion: from-csv
* Sampling: sample

Notes:
* Run `tva help <SUBCOMMAND>` for detailed usage
* Run `tva --help-fields` for shared field syntax used by select/join/uniq/split
"###,
        );

    let matches = app.get_matches();

    if matches.get_flag("help-fields") {
        use tva::libs::fields::FIELD_SYNTAX_HELP;
        println!("{}", FIELD_SYNTAX_HELP);
        return Ok(());
    }

    match matches.subcommand() {
        Some(("md", sub_matches)) => cmd_tva::md::execute(sub_matches),
        Some(("append", sub_matches)) => cmd_tva::append::execute(sub_matches),
        Some(("join", sub_matches)) => cmd_tva::join::execute(sub_matches),
        Some(("uniq", sub_matches)) => cmd_tva::uniq::execute(sub_matches),
        Some(("nl", sub_matches)) => cmd_tva::nl::execute(sub_matches),
        Some(("keep-header", sub_matches)) => cmd_tva::keep_header::execute(sub_matches),
        Some(("longer", sub_matches)) => cmd_tva::longer::execute(sub_matches),
        Some(("check", sub_matches)) => cmd_tva::check::execute(sub_matches),
        Some(("transpose", sub_matches)) => cmd_tva::transpose::execute(sub_matches),
        Some(("sort", sub_matches)) => cmd_tva::sort::execute(sub_matches),
        Some(("from-csv", sub_matches)) => cmd_tva::from_csv::execute(sub_matches),
        Some(("select", sub_matches)) => cmd_tva::select::execute(sub_matches),
        Some(("sample", sub_matches)) => cmd_tva::sample::execute(sub_matches),
        Some(("split", sub_matches)) => cmd_tva::split::execute(sub_matches),
        Some(("filter", sub_matches)) => cmd_tva::filter::execute(sub_matches),
        Some(("stats", sub_matches)) => cmd_tva::stats::execute(sub_matches),
        _ => unreachable!(),
    }?;

    Ok(())
}

// TODO: `rgr span` 5p and 3p
// TODO: --bed for `rgr field`
