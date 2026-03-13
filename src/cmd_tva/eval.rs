use clap::*;

use crate::libs::expr::{parser, runtime};

pub fn make_subcommand() -> Command {
    Command::new("eval")
        .about("Evaluates an expression for testing/debugging")
        .after_help(include_str!("../../docs/help/eval.md"))
        .arg(
            Arg::new("expr")
                .index(1)
                .required(true)
                .help("Expression to evaluate (e.g., '@1 + @2', 'upper(@name)')"),
        )
        .arg(
            Arg::new("headers")
                .long("headers")
                .short('H')
                .num_args(1)
                .help("Comma-separated header names for column reference (e.g., 'name,age')"),
        )
        .arg(
            Arg::new("row")
                .long("row")
                .short('r')
                .action(ArgAction::Append)
                .help("Comma-separated row values to evaluate against (e.g., 'Alice,30'). Can be specified multiple times for multiple rows."),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let expr_str = args.get_one::<String>("expr").unwrap();
    let headers_str = args.get_one::<String>("headers");
    let row_values: Vec<String> = match args.get_many::<String>("row") {
        Some(values) => values.cloned().collect(),
        None => Vec::new(),
    };

    // Parse the expression
    let parsed_expr = parser::parse(expr_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse expression: {}", e))?;

    // Build headers if provided
    let headers: Option<Vec<String>> = headers_str
        .as_ref()
        .map(|h| h.split(',').map(|s| s.trim().to_string()).collect());

    // Process each row
    if row_values.is_empty() {
        // No rows provided, evaluate with empty context
        let row: Vec<String> = headers
            .as_ref()
            .map(|h| vec!["".to_string(); h.len()])
            .unwrap_or_default();
        let mut ctx = match headers.as_ref() {
            Some(h) => runtime::EvalContext::with_headers(&row, h),
            None => runtime::EvalContext::new(&row),
        };
        let result = runtime::eval(&parsed_expr, &mut ctx)
            .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;
        println!("{}", result.to_string());
    } else {
        for row_str in &row_values {
            let row: Vec<String> =
                row_str.split(',').map(|s| s.trim().to_string()).collect();
            let mut ctx = match headers.as_ref() {
                Some(h) => runtime::EvalContext::with_headers(&row, h),
                None => runtime::EvalContext::new(&row),
            };
            let result = runtime::eval(&parsed_expr, &mut ctx)
                .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;
            println!("{}", result.to_string());
        }
    }

    Ok(())
}
