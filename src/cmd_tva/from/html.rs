use clap::{Arg, ArgMatches, Command};
use scraper::{ElementRef, Html, Selector};
use std::io::{Read, Write};

pub fn make_subcommand() -> Command {
    Command::new("html")
        .about("Extract data from HTML")
        .after_help(include_str!("../../../docs/help/from_html.md"))
        .arg(
            Arg::new("infile")
                .num_args(0..=1)
                .default_value("stdin")
                .index(1)
                .help("Input HTML file to process (default: stdin)"),
        )
        .arg(
            Arg::new("query")
                .long("query")
                .short('q')
                .value_name("QUERY")
                .num_args(1)
                .conflicts_with_all(["table", "index", "row", "col"])
                .help("Query string (e.g. \"a attr{href}\")"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .arg(
            Arg::new("table")
                .long("table")
                .value_name("SELECTOR")
                .num_args(0..=1)
                .default_missing_value("table")
                .require_equals(true)
                .conflicts_with_all(["query", "row", "col"])
                .help("Extract HTML table(s) to TSV"),
        )
        .arg(
            Arg::new("index")
                .long("index")
                .value_name("N")
                .value_parser(clap::value_parser!(usize))
                .conflicts_with_all(["query", "row", "col"])
                .help("Select the N-th matched table (1-based). Implies --table"),
        )
        .arg(
            Arg::new("row")
                .long("row")
                .value_name("SELECTOR")
                .conflicts_with_all(["query", "table"])
                .help("Row selector for list mode (each match -> one TSV row)"),
        )
        .arg(
            Arg::new("col")
                .long("col")
                .value_name("NAME:SELECTOR FUNC")
                .action(clap::ArgAction::Append)
                .requires("row")
                .conflicts_with_all(["query", "table"])
                .help("Column spec: 'Name:Selector Func' (Func: text{}|attr{name})"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Read input
    let infile = args.get_one::<String>("infile").unwrap();
    let mut reader = crate::libs::io::raw_reader(infile);
    let mut html_content = String::new();
    reader.read_to_string(&mut html_content)?;

    // 1b. Prepare output
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    // 2. Parse HTML
    let document = Html::parse_document(&html_content);

    // 3. Check for --table mode (explicit or implied by --index)
    if args.contains_id("table") || args.get_one::<usize>("index").is_some() {
        let tsv = extract_table(args, &document)?;
        writer.write_all(tsv.as_bytes())?;
        return Ok(());
    }

    // 4. Check for --row (List Extraction) mode
    if let Some(row_selector) = args.get_one::<String>("row") {
        let tsv = extract_struct(args, &document, row_selector)?;
        writer.write_all(tsv.as_bytes())?;
        return Ok(());
    }

    // 5. Parse Selectors & Display Function (Pup Mode)
    let query_opt = args
        .get_one::<String>("query")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let query = if let Some(q) = query_opt {
        q
    } else {
        writer.write_all(html_content.as_bytes())?;
        return Ok(());
    };

    let out = extract_query(&document, &query)?;
    writer.write_all(out.as_bytes())?;

    Ok(())
}

fn parse_query(query: &str) -> (String, Option<String>) {
    let tokens: Vec<&str> = query.split_whitespace().collect();

    if tokens.is_empty() {
        return (String::new(), None);
    }

    let last = tokens[tokens.len() - 1];
    if is_display_function(last) {
        let selector = tokens[..tokens.len() - 1].join(" ");
        (selector, Some(last.to_string()))
    } else {
        (tokens.join(" "), None)
    }
}

fn is_display_function(s: &str) -> bool {
    s == "text{}"
        || s == "text()"
        || (s.starts_with("attr{") && s.ends_with("}"))
        || (s.starts_with("attr(") && s.ends_with(")"))
}

fn extract_query(document: &Html, query: &str) -> anyhow::Result<String> {
    let (selector_str, display_func) = parse_query(query);
    let selector = Selector::parse(&selector_str)
        .map_err(|e| anyhow::anyhow!("Invalid CSS selector: {:?}", e))?;
    let selected = document.select(&selector);
    let mut out = String::new();
    match display_func.as_deref() {
        Some("text{}") | Some("text()") => {
            for element in selected {
                let text = element.text().collect::<Vec<_>>().join("");
                out.push_str(&text);
                out.push('\n');
            }
        }
        Some(s) if s.starts_with("attr{") && s.ends_with("}") => {
            let attr = &s[5..s.len() - 1];
            for element in selected {
                if let Some(val) = element.value().attr(attr) {
                    out.push_str(val);
                }
                out.push('\n');
            }
        }
        Some(s) if s.starts_with("attr(") && s.ends_with(")") => {
            let attr = s[5..s.len() - 1].trim_matches(|c| c == '"' || c == '\'');
            for element in selected {
                if let Some(val) = element.value().attr(attr) {
                    out.push_str(val);
                }
                out.push('\n');
            }
        }
        _ => {
            for element in selected {
                out.push_str(&element.html());
                out.push('\n');
            }
        }
    }
    Ok(out)
}

fn extract_table(args: &ArgMatches, document: &Html) -> anyhow::Result<String> {
    let table_selector_str = args
        .get_one::<String>("table")
        .map(|s| s.as_str())
        .unwrap_or("table");

    let table_selector = Selector::parse(&table_selector_str)
        .map_err(|e| anyhow::anyhow!("Invalid table selector: {:?}", e))?;

    let mut tables = document.select(&table_selector);

    // Handle --index
    let table_element = if let Some(index) = args.get_one::<usize>("index") {
        if *index == 0 {
            return Err(anyhow::anyhow!("Index must be >= 1"));
        }
        tables.nth(*index - 1)
    } else {
        tables.next()
    };

    let table = match table_element {
        Some(t) => t,
        None => {
            return Err(anyhow::anyhow!(
                "No table found matching '{}'",
                table_selector_str
            ))
        }
    };

    let mut buf: Vec<u8> = Vec::new();
    let mut writer = ::csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(&mut buf);

    // Iterate over children to avoid selecting nested tables' rows
    // Standard structure: table -> [caption, colgroup, thead, tbody, tfoot, tr]
    // We care about thead, tbody, tfoot, and direct tr

    for child in table.children() {
        if let Some(element) = ElementRef::wrap(child) {
            match element.value().name() {
                "thead" | "tbody" | "tfoot" => {
                    for row_child in element.children() {
                        if let Some(row) = ElementRef::wrap(row_child) {
                            if row.value().name() == "tr" {
                                process_row(row, &mut writer)?;
                            }
                        }
                    }
                }
                "tr" => {
                    process_row(element, &mut writer)?;
                }
                _ => {}
            }
        }
    }

    writer.flush()?;
    drop(writer);
    String::from_utf8(buf).map_err(|e| anyhow::anyhow!("invalid UTF-8: {}", e))
}

fn process_row<W: std::io::Write>(
    row: ElementRef,
    writer: &mut ::csv::Writer<W>,
) -> anyhow::Result<()> {
    let mut cells = Vec::new();
    for child in row.children() {
        if let Some(cell) = ElementRef::wrap(child) {
            match cell.value().name() {
                "td" | "th" => {
                    let text =
                        cell.text().collect::<Vec<_>>().join("").trim().to_string();
                    cells.push(text);
                }
                _ => {}
            }
        }
    }

    if !cells.is_empty() {
        writer.write_record(&cells)?;
    }
    Ok(())
}

fn extract_struct(
    args: &ArgMatches,
    document: &Html,
    row_selector_str: &str,
) -> anyhow::Result<String> {
    let row_selector = Selector::parse(row_selector_str)
        .map_err(|e| anyhow::anyhow!("Invalid row selector: {:?}", e))?;

    let col_defs: Vec<&String> = args
        .get_many::<String>("col")
        .ok_or_else(|| anyhow::anyhow!("--col is required when using --row"))?
        .collect();

    let mut columns = Vec::new();
    for def in col_defs {
        // Parse "Name:Selector Func"
        // Find first colon
        let parts: Vec<&str> = def.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid column definition: '{}'. Expected format 'Name:Selector Func'",
                def
            ));
        }
        let name = parts[0].trim();
        let rest = parts[1].trim();

        let (selector_str, func) = parse_col_func(rest)?;
        let selector = if selector_str.is_empty() {
            None
        } else {
            Some(Selector::parse(selector_str).map_err(|e| {
                anyhow::anyhow!("Invalid column selector '{}': {:?}", selector_str, e)
            })?)
        };

        columns.push((name.to_string(), selector, func));
    }

    let mut buf: Vec<u8> = Vec::new();
    let mut writer = ::csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(&mut buf);

    // Write Header
    let headers: Vec<String> = columns.iter().map(|c| c.0.clone()).collect();
    writer.write_record(&headers)?;

    // Write Rows
    for row_node in document.select(&row_selector) {
        let mut cells = Vec::new();
        for (_, selector, func) in &columns {
            let value = if let Some(sel) = selector {
                if let Some(el) = row_node.select(sel).next() {
                    extract_value(el, func)?
                } else {
                    String::new()
                }
            } else {
                extract_value(row_node, func)?
            };
            cells.push(value);
        }
        writer.write_record(&cells)?;
    }

    writer.flush()?;
    drop(writer);
    String::from_utf8(buf).map_err(|e| anyhow::anyhow!("invalid UTF-8: {}", e))
}

enum ColFunc {
    Text,
    Attr(String),
}

fn parse_col_func(s: &str) -> anyhow::Result<(&str, ColFunc)> {
    let s = s.trim();
    if s.ends_with("text()") || s.ends_with("text{}") {
        Ok((s[..s.len() - 6].trim(), ColFunc::Text))
    } else if let Some(idx) = s.rfind("attr(") {
        if s.ends_with(")") {
            let attr_part = &s[idx..];
            // attr("href") -> href
            // handle quotes inside parens
            let inner = &attr_part[5..attr_part.len() - 1];
            let attr_name = inner.trim().trim_matches(|c| c == '"' || c == '\'');
            Ok((s[..idx].trim(), ColFunc::Attr(attr_name.to_string())))
        } else {
            Err(anyhow::anyhow!("Invalid attr() syntax in '{}'", s))
        }
    } else if let Some(idx) = s.rfind("attr{") {
        if s.ends_with("}") {
            let attr_part = &s[idx..];
            let inner = &attr_part[5..attr_part.len() - 1];
            let attr_name = inner.trim().trim_matches(|c| c == '"' || c == '\'');
            Ok((s[..idx].trim(), ColFunc::Attr(attr_name.to_string())))
        } else {
            Err(anyhow::anyhow!("Invalid attr{{}} syntax in '{}'", s))
        }
    } else {
        // Default to Text if no func specified? Or error?
        // Let's default to Text for usability.
        Ok((s, ColFunc::Text))
    }
}

fn extract_value(element: ElementRef, func: &ColFunc) -> anyhow::Result<String> {
    match func {
        ColFunc::Text => Ok(element
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string()),
        ColFunc::Attr(name) => Ok(element.value().attr(name).unwrap_or("").to_string()),
    }
}
