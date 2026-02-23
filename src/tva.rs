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
        .subcommand(cmd_tva::md::make_subcommand())
        .subcommand(cmd_tva::dedup::make_subcommand())
        .subcommand(cmd_tva::nl::make_subcommand())
        .subcommand(cmd_tva::keep_header::make_subcommand())
        .subcommand(cmd_tva::check::make_subcommand())
        .subcommand(cmd_tva::transpose::make_subcommand())
        .subcommand(cmd_tva::sort::make_subcommand())
        .after_help(
            r###"
Tab-separated Values Assistant (tva): small toolbox for working with TSV files.

Currently implemented subcommands:
* Generic TSV: md, dedup, nl, transpose, sort
* Table plumbing: keep-header, check

Notes:
* Run `tva help <SUBCOMMAND>` for detailed usage
"###,
        );

    match app.get_matches().subcommand() {
        Some(("md", sub_matches)) => cmd_tva::md::execute(sub_matches),
        Some(("dedup", sub_matches)) => cmd_tva::dedup::execute(sub_matches),
        Some(("nl", sub_matches)) => cmd_tva::nl::execute(sub_matches),
        Some(("keep-header", sub_matches)) => cmd_tva::keep_header::execute(sub_matches),
        Some(("check", sub_matches)) => cmd_tva::check::execute(sub_matches),
        Some(("transpose", sub_matches)) => cmd_tva::transpose::execute(sub_matches),
        Some(("sort", sub_matches)) => cmd_tva::sort::execute(sub_matches),
        _ => unreachable!(),
    }
    .unwrap();

    Ok(())
}

// TODO: `rgr span` 5p and 3p
// TODO: --bed for `rgr field`
