use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn trim(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().trim().to_string())),
    }
}

pub fn upper(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().to_uppercase())),
    }
}

pub fn lower(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().to_lowercase())),
    }
}

pub fn len(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(list) => Ok(Value::Int(list.len() as i64)),
        Value::Null => Ok(Value::Int(0)),
        v => Ok(Value::Int(v.to_string().len() as i64)),
    }
}

pub fn substr(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let start = args[1].as_int().unwrap_or(0) as usize;
    let len = args[2].as_int().unwrap_or(s.len() as i64) as usize;

    if start >= s.len() {
        return Ok(Value::String(String::new()));
    }

    let end = (start + len).min(s.len());
    Ok(Value::String(s[start..end].to_string()))
}

pub fn split(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let delim = args[1].as_string();

    let parts: Vec<Value> = s
        .split(&delim)
        .map(|p| Value::String(p.to_string()))
        .collect();
    Ok(Value::List(parts))
}

pub fn contains(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let substr = args[1].as_string();
    Ok(Value::Bool(s.contains(&substr)))
}

pub fn starts_with(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let prefix = args[1].as_string();
    Ok(Value::Bool(s.starts_with(&prefix)))
}

pub fn ends_with(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let suffix = args[1].as_string();
    Ok(Value::Bool(s.ends_with(&suffix)))
}

pub fn replace(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let from = args[1].as_string();
    let to = args[2].as_string();
    Ok(Value::String(s.replace(&from, &to)))
}

pub fn wordcount(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let count = s.split_whitespace().count() as i64;
    Ok(Value::Int(count))
}

pub fn char_len(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    Ok(Value::Int(s.chars().count() as i64))
}

pub fn truncate(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let len = match &args[1] {
        Value::Int(n) => *n as usize,
        Value::Float(f) => f.round() as usize,
        v => {
            return Err(EvalError::TypeError(format!(
                "truncate: length must be a number, got {}",
                v.type_name()
            )))
        }
    };
    let end = if args.len() > 2 {
        args[2].as_string()
    } else {
        "...".to_string()
    };

    if s.len() <= len {
        Ok(Value::String(s))
    } else {
        let truncated = &s[..len.saturating_sub(end.len())];
        Ok(Value::String(format!("{}{}", truncated, end)))
    }
}

/// Format a string using %() placeholders.
/// Supports three delimiter types: %(), %[], %{}
/// Format specifiers follow Rust's format! syntax.
pub fn fmt(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "fmt".to_string(),
            expected: 1,
            got: 0,
        });
    }

    let template = args[0].as_string();
    let format_args = &args[1..];

    // Parse and format the template (no context for basic fmt)
    match format_template(&template, format_args, None, None, None, None) {
        Ok(result) => Ok(Value::String(result)),
        Err(e) => Err(EvalError::TypeError(format!("fmt: {}", e))),
    }
}

/// Format with context support (for %(@n) and %(var) placeholders).
/// This version is called from eval() when fmt is invoked with context.
pub fn fmt_with_context(
    args: &[Value],
    row: Option<&[String]>,
    variables: Option<&ahash::HashMap<String, Value>>,
    lambda_params: Option<&ahash::HashMap<String, Value>>,
    globals: Option<std::cell::Ref<'_, ahash::HashMap<String, Value>>>,
) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "fmt".to_string(),
            expected: 1,
            got: 0,
        });
    }

    let template = args[0].as_string();
    let format_args = &args[1..];

    // Parse and format the template with context
    match format_template(
        &template,
        format_args,
        row,
        variables,
        lambda_params,
        globals,
    ) {
        Ok(result) => Ok(Value::String(result)),
        Err(e) => Err(EvalError::TypeError(format!("fmt: {}", e))),
    }
}

/// Format a template string with the given arguments.
fn format_template(
    template: &str,
    args: &[Value],
    row: Option<&[String]>,
    variables: Option<&ahash::HashMap<String, Value>>,
    lambda_params: Option<&ahash::HashMap<String, Value>>,
    globals: Option<std::cell::Ref<'_, ahash::HashMap<String, Value>>>,
) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    let mut arg_index = 0;

    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Check for escaped %%
            if chars.peek() == Some(&'%') {
                chars.next();
                result.push('%');
                continue;
            }

            // Parse the placeholder
            match parse_placeholder(
                &mut chars,
                args,
                &mut arg_index,
                row,
                lambda_params,
            )? {
                Placeholder::NextArg => {
                    if arg_index >= args.len() {
                        return Err(format!(
                            "not enough arguments: expected at least {}, got {}",
                            arg_index + 1,
                            args.len()
                        ));
                    }
                    result.push_str(&value_to_string(&args[arg_index]));
                    arg_index += 1;
                }
                Placeholder::IndexedArg(idx) => {
                    let idx = idx.saturating_sub(1); // Convert 1-based to 0-based
                    if idx >= args.len() {
                        return Err(format!(
                            "argument index {} out of range (got {} arguments)",
                            idx + 1,
                            args.len()
                        ));
                    }
                    result.push_str(&value_to_string(&args[idx]));
                }
                Placeholder::FormattedNextArg(spec) => {
                    if arg_index >= args.len() {
                        return Err(format!(
                            "not enough arguments: expected at least {}, got {}",
                            arg_index + 1,
                            args.len()
                        ));
                    }
                    result.push_str(&format_value(&args[arg_index], &spec)?);
                    arg_index += 1;
                }
                Placeholder::FormattedIndexedArg(idx, spec) => {
                    let idx = idx.saturating_sub(1); // Convert 1-based to 0-based
                    if idx >= args.len() {
                        return Err(format!(
                            "argument index {} out of range (got {} arguments)",
                            idx + 1,
                            args.len()
                        ));
                    }
                    result.push_str(&format_value(&args[idx], &spec)?);
                }
                Placeholder::ColumnRef(idx) => {
                    let idx = idx.saturating_sub(1); // Convert 1-based to 0-based
                    match row {
                        Some(r) if idx < r.len() => {
                            result.push_str(&r[idx]);
                        }
                        _ => {
                            return Err(format!(
                                "column index {} out of range",
                                idx + 1
                            ));
                        }
                    }
                }
                Placeholder::VarRef(name) => {
                    // First check lambda_params, then variables, then globals
                    let value = lambda_params
                        .and_then(|params| params.get(&name))
                        .cloned()
                        .or_else(|| variables.and_then(|vars| vars.get(&name)).cloned())
                        .or_else(|| globals.as_ref().and_then(|g| g.get(&name)).cloned())
                        .unwrap_or_else(|| Value::Null);
                    result.push_str(&value_to_string(&value));
                }
                Placeholder::FormattedColumnRef(idx, spec) => {
                    let idx = idx.saturating_sub(1); // Convert 1-based to 0-based
                    match row {
                        Some(r) if idx < r.len() => {
                            let val = parse_value(&r[idx]);
                            result.push_str(&format_value(&val, &spec)?);
                        }
                        _ => {
                            return Err(format!(
                                "column index {} out of range",
                                idx + 1
                            ));
                        }
                    }
                }
                Placeholder::FormattedVarRef(name, spec) => {
                    // First check lambda_params, then variables, then globals
                    let value = lambda_params
                        .and_then(|params| params.get(&name))
                        .cloned()
                        .or_else(|| variables.and_then(|vars| vars.get(&name)).cloned())
                        .or_else(|| globals.as_ref().and_then(|g| g.get(&name)).cloned())
                        .unwrap_or_else(|| Value::Null);
                    result.push_str(&format_value(&value, &spec)?);
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

/// Parse a string value into Value (try int, then float, then string)
fn parse_value(s: &str) -> Value {
    if s.is_empty() {
        return Value::Null;
    }

    // Try integer first
    if let Ok(i) = s.parse::<i64>() {
        return Value::Int(i);
    }

    // Then float
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }

    // Fall back to string
    Value::String(s.to_string())
}

/// Represents a parsed placeholder.
enum Placeholder {
    NextArg,                            // %() or %[] or %{}
    IndexedArg(usize),                  // %(n) or %[n] or %{n}
    FormattedNextArg(String),           // %(:spec) or %[:spec] or %{:spec}
    FormattedIndexedArg(usize, String), // %(n:spec) or %[n:spec] or %{n:spec}
    ColumnRef(usize),                   // %(@n) - column by index
    VarRef(String),                     // %(var) or %(@var) - variable reference
    FormattedColumnRef(usize, String),  // %(@n:spec) - column with format
    FormattedVarRef(String, String), // %(var:spec) or %(@var:spec) - variable with format
}

/// Parse a placeholder starting after the '%' character.
fn parse_placeholder(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    _args: &[Value],
    _current_index: &mut usize,
    _row: Option<&[String]>,
    _lambda_params: Option<&ahash::HashMap<String, Value>>,
) -> Result<Placeholder, String> {
    // Determine the delimiter type
    let (open_delim, close_delim) = match chars.peek() {
        Some(&'(') => {
            chars.next();
            ('(', ')')
        }
        Some(&'[') => {
            chars.next();
            ('[', ']')
        }
        Some(&'{') => {
            chars.next();
            ('{', '}')
        }
        Some(&ch) => return Err(format!("expected delimiter after %, found '{}'", ch)),
        None => return Err("unexpected end of template after %".to_string()),
    };

    // Parse the content inside the delimiters
    let mut content = String::new();
    let mut depth = 1;

    while let Some(ch) = chars.next() {
        if ch == open_delim {
            depth += 1;
            content.push(ch);
        } else if ch == close_delim {
            depth -= 1;
            if depth == 0 {
                break;
            }
            content.push(ch);
        } else {
            content.push(ch);
        }
    }

    if depth != 0 {
        return Err(format!("unclosed delimiter '{}'", open_delim));
    }

    // Parse the content: could be empty, a number, @n, var, or with format_spec
    if content.is_empty() {
        // %() - next argument
        Ok(Placeholder::NextArg)
    } else if let Some(colon_pos) = content.find(':') {
        // Has format specifier
        let (index_part, spec) = content.split_at(colon_pos);
        let spec = &spec[1..]; // Remove the leading ':'

        if index_part.is_empty() {
            // %(:spec) - next argument with format
            Ok(Placeholder::FormattedNextArg(spec.to_string()))
        } else if index_part.starts_with('@') {
            // %(@n:spec) or %(@var:spec)
            let name = &index_part[1..];
            if let Ok(idx) = name.parse::<usize>() {
                Ok(Placeholder::FormattedColumnRef(idx, spec.to_string()))
            } else {
                Ok(Placeholder::FormattedVarRef(
                    name.to_string(),
                    spec.to_string(),
                ))
            }
        } else if let Ok(idx) = index_part.parse::<usize>() {
            // %(n:spec) - indexed argument with format
            Ok(Placeholder::FormattedIndexedArg(idx, spec.to_string()))
        } else {
            // %(var:spec) - variable reference with format
            Ok(Placeholder::FormattedVarRef(
                index_part.to_string(),
                spec.to_string(),
            ))
        }
    } else if content.starts_with('@') {
        // %(@n) or %(@var) - column or variable reference
        let name = &content[1..];
        if let Ok(idx) = name.parse::<usize>() {
            Ok(Placeholder::ColumnRef(idx))
        } else {
            Ok(Placeholder::VarRef(name.to_string()))
        }
    } else if let Ok(idx) = content.parse::<usize>() {
        // %(n) - indexed argument
        Ok(Placeholder::IndexedArg(idx))
    } else {
        // %(var) - variable reference
        Ok(Placeholder::VarRef(content.to_string()))
    }
}

/// Convert a Value to its string representation.
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::List(_) => "[list]".to_string(),
        Value::DateTime(dt) => dt.to_rfc3339(),
        Value::Lambda(_) => "[lambda]".to_string(),
    }
}

/// Format a value according to the format specification.
fn format_value(value: &Value, spec: &str) -> Result<String, String> {
    // Parse the format spec
    let mut fill = ' ';
    let mut align = '<';
    let mut width = None::<usize>;
    let mut precision = None::<usize>;
    let mut sign = '-';
    let mut alternate = false;
    let ty;

    let mut chars = spec.chars().peekable();

    // Parse fill and align
    if let Some(&ch) = chars.peek() {
        if let Some(next) = chars.clone().nth(1) {
            if next == '<' || next == '>' || next == '^' {
                fill = ch;
                chars.next();
                align = chars.next().unwrap();
            } else if ch == '<' || ch == '>' || ch == '^' {
                align = ch;
                chars.next();
            }
        } else if ch == '<' || ch == '>' || ch == '^' {
            align = ch;
            chars.next();
        }
    }

    // Parse sign
    if let Some(&'+') = chars.peek() {
        sign = '+';
        chars.next();
    } else if let Some(&'-') = chars.peek() {
        sign = '-';
        chars.next();
    }

    // Parse alternate form
    if let Some(&'#') = chars.peek() {
        alternate = true;
        chars.next();
    }

    // Parse width
    let mut width_str = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            width_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    if !width_str.is_empty() {
        width = Some(width_str.parse().map_err(|_| "invalid width")?);
    }

    // Parse precision
    if let Some(&'.') = chars.peek() {
        chars.next();
        let mut prec_str = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                prec_str.push(ch);
                chars.next();
            } else {
                break;
            }
        }
        if !prec_str.is_empty() {
            precision = Some(prec_str.parse().map_err(|_| "invalid precision")?);
        }
    }

    // Parse type
    let remaining: String = chars.collect();
    ty = remaining.as_str();

    // Format the value
    let formatted = match value {
        Value::Int(n) => format_int(*n, sign, alternate, width, precision, ty)?,
        Value::Float(f) => format_float(*f, sign, alternate, width, precision, ty)?,
        Value::String(s) => format_string(s, width, precision)?,
        v => value_to_string(v),
    };

    // Apply alignment and fill
    if let Some(w) = width {
        let len = formatted.chars().count();
        if len < w {
            let pad = w - len;
            match align {
                '<' => Ok(format!("{}{}", formatted, fill.to_string().repeat(pad))),
                '>' => Ok(format!("{}{}", fill.to_string().repeat(pad), formatted)),
                '^' => {
                    let left = pad / 2;
                    let right = pad - left;
                    Ok(format!(
                        "{}{}{}",
                        fill.to_string().repeat(left),
                        formatted,
                        fill.to_string().repeat(right)
                    ))
                }
                _ => Ok(formatted),
            }
        } else {
            Ok(formatted)
        }
    } else {
        Ok(formatted)
    }
}

/// Format an integer value.
fn format_int(
    n: i64,
    sign: char,
    alternate: bool,
    width: Option<usize>,
    precision: Option<usize>,
    ty: &str,
) -> Result<String, String> {
    let mut result = match ty {
        "b" => {
            let prefix = if alternate { "0b" } else { "" };
            format!("{}{:b}", prefix, n)
        }
        "o" => {
            let prefix = if alternate { "0o" } else { "" };
            format!("{}{:o}", prefix, n)
        }
        "x" => {
            let prefix = if alternate { "0x" } else { "" };
            format!("{}{:x}", prefix, n)
        }
        "X" => {
            let prefix = if alternate { "0X" } else { "" };
            format!("{}{:X}", prefix, n)
        }
        "" | "d" | "i" => {
            if let Some(prec) = precision {
                format!("{:0>prec$}", n, prec = prec)
            } else {
                n.to_string()
            }
        }
        "e" => format!("{:e}", n as f64),
        "E" => format!("{:E}", n as f64),
        "?" => format!("{:?}", n),
        _ => return Err(format!("unknown format type '{}' for integer", ty)),
    };

    // Apply sign
    if sign == '+' && n >= 0 && !result.starts_with('+') {
        result = format!("+{}", result);
    }

    // Apply zero-padding for width if specified
    if let Some(w) = width {
        if result.len() < w && !result.starts_with('-') && !result.starts_with('+') {
            result = format!("{:0>width$}", result, width = w);
        } else if result.len() < w {
            // Handle sign separately
            let sign_char = if result.starts_with('-') {
                "-"
            } else if result.starts_with('+') {
                "+"
            } else {
                ""
            };
            let num_part = &result[sign_char.len()..];
            result = format!("{}{:0>width$}", sign_char, num_part, width = w - 1);
        }
    }

    Ok(result)
}

/// Format a float value.
fn format_float(
    f: f64,
    sign: char,
    _alternate: bool,
    _width: Option<usize>,
    precision: Option<usize>,
    ty: &str,
) -> Result<String, String> {
    let prec = precision.unwrap_or(6);

    let mut result = match ty {
        "e" => format!("{:.*e}", prec, f),
        "E" => format!("{:.*E}", prec, f),
        "" | "f" | "g" | "G" => format!("{:.*}", prec, f),
        "?" => format!("{:?}", f),
        _ => return Err(format!("unknown format type '{}' for float", ty)),
    };

    // Apply sign
    if sign == '+' && f >= 0.0 && !result.starts_with('+') {
        result = format!("+{}", result);
    }

    Ok(result)
}

/// Format a string value.
fn format_string(
    s: &str,
    _width: Option<usize>,
    precision: Option<usize>,
) -> Result<String, String> {
    let mut result = s.to_string();

    // Apply precision (truncate)
    if let Some(prec) = precision {
        let chars: Vec<char> = result.chars().collect();
        if chars.len() > prec {
            result = chars[..prec].iter().collect();
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() {
        assert_eq!(
            trim(&[Value::String("  hello  ".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_upper() {
        assert_eq!(
            upper(&[Value::String("hello".to_string())]).unwrap(),
            Value::String("HELLO".to_string())
        );
    }

    #[test]
    fn test_lower() {
        assert_eq!(
            lower(&[Value::String("HELLO".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_len() {
        assert_eq!(
            len(&[Value::String("hello".to_string())]).unwrap(),
            Value::Int(5)
        );
        // Null returns 0
        assert_eq!(len(&[Value::Null]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_substr() {
        assert_eq!(
            substr(&[
                Value::String("hello world".to_string()),
                Value::Int(0),
                Value::Int(5),
            ])
            .unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_split() {
        match split(&[
            Value::String("a,b,c".to_string()),
            Value::String(",".to_string()),
        ])
        .unwrap()
        {
            Value::List(vals) => {
                assert_eq!(vals.len(), 3);
                assert_eq!(vals[0], Value::String("a".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_contains() {
        assert_eq!(
            contains(&[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_starts_with() {
        assert_eq!(
            starts_with(&[
                Value::String("hello".to_string()),
                Value::String("he".to_string()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_ends_with() {
        assert_eq!(
            ends_with(&[
                Value::String("hello".to_string()),
                Value::String("lo".to_string()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            replace(&[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
                Value::String("rust".to_string()),
            ])
            .unwrap(),
            Value::String("hello rust".to_string())
        );
    }

    #[test]
    fn test_truncate() {
        // "hello world" is 11 bytes, limit 8, end "..." is 3 bytes
        assert_eq!(
            truncate(&[Value::String("hello world".to_string()), Value::Int(8)])
                .unwrap(),
            Value::String("hello...".to_string())
        );

        // Custom ending
        assert_eq!(
            truncate(&[
                Value::String("hello world".to_string()),
                Value::Int(8),
                Value::String(">>".to_string()),
            ])
            .unwrap(),
            Value::String("hello >>".to_string())
        );

        // String shorter than limit - return as-is
        assert_eq!(
            truncate(&[Value::String("hi".to_string()), Value::Int(10)]).unwrap(),
            Value::String("hi".to_string())
        );
    }

    #[test]
    fn test_wordcount() {
        assert_eq!(
            wordcount(&[Value::String("hello world foo bar".to_string())]).unwrap(),
            Value::Int(4)
        );
        assert_eq!(
            wordcount(&[Value::String("   multiple   spaces   ".to_string())]).unwrap(),
            Value::Int(2)
        );
        assert_eq!(
            wordcount(&[Value::String("".to_string())]).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_char_len() {
        assert_eq!(
            char_len(&[Value::String("hello".to_string())]).unwrap(),
            Value::Int(5)
        );
        // UTF-8 characters
        assert_eq!(
            char_len(&[Value::String("你好世界".to_string())]).unwrap(),
            Value::Int(4)
        );
    }

    #[test]
    fn test_trim_null() {
        assert_eq!(trim(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_len_null() {
        assert_eq!(len(&[Value::Null]).unwrap(), Value::Int(0));
    }

    // Additional tests to improve coverage

    #[test]
    fn test_trim_non_string() {
        // trim with non-string, non-null value
        assert_eq!(
            trim(&[Value::Int(123)]).unwrap(),
            Value::String("123".to_string())
        );
        assert_eq!(
            trim(&[Value::Bool(true)]).unwrap(),
            Value::String("true".to_string())
        );
    }

    #[test]
    fn test_upper_null() {
        assert_eq!(upper(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_upper_non_string() {
        assert_eq!(
            upper(&[Value::Int(123)]).unwrap(),
            Value::String("123".to_string())
        );
    }

    #[test]
    fn test_lower_null() {
        assert_eq!(lower(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_lower_non_string() {
        assert_eq!(
            lower(&[Value::Int(123)]).unwrap(),
            Value::String("123".to_string())
        );
    }

    #[test]
    fn test_len_list() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(len(&[list]).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_len_non_string() {
        assert_eq!(len(&[Value::Int(12345)]).unwrap(), Value::Int(5));
        assert_eq!(len(&[Value::Bool(true)]).unwrap(), Value::Int(4));
    }

    #[test]
    fn test_substr_start_beyond_length() {
        // When start >= string length, return empty string
        assert_eq!(
            substr(&[
                Value::String("hello".to_string()),
                Value::Int(10),
                Value::Int(5),
            ])
            .unwrap(),
            Value::String("".to_string())
        );
    }

    #[test]
    fn test_truncate_with_float() {
        // truncate with Float length
        // "hello world" (11 chars), limit 8, end "..." (3 chars)
        // truncated = 8 - 3 = 5 chars from original = "hello" + "..." = "hello..."
        assert_eq!(
            truncate(&[Value::String("hello world".to_string()), Value::Float(8.0)])
                .unwrap(),
            Value::String("hello...".to_string())
        );
        // Float rounding: 8.7 rounds to 9
        // truncated = 9 - 3 = 6 chars from original = "hello " + "..." = "hello ..."
        assert_eq!(
            truncate(&[Value::String("hello world".to_string()), Value::Float(8.7)])
                .unwrap(),
            Value::String("hello ...".to_string())
        );
    }

    #[test]
    fn test_truncate_type_error() {
        // truncate with non-number should return error
        let result = truncate(&[
            Value::String("hello".to_string()),
            Value::String("5".to_string()),
        ]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("truncate: length must be a number"));
    }

    #[test]
    fn test_contains_not_found() {
        assert_eq!(
            contains(&[
                Value::String("hello world".to_string()),
                Value::String("foo".to_string()),
            ])
            .unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_starts_with_false() {
        assert_eq!(
            starts_with(&[
                Value::String("hello".to_string()),
                Value::String("lo".to_string()),
            ])
            .unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_ends_with_false() {
        assert_eq!(
            ends_with(&[
                Value::String("hello".to_string()),
                Value::String("he".to_string()),
            ])
            .unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_split_empty() {
        match split(&[
            Value::String("".to_string()),
            Value::String(",".to_string()),
        ])
        .unwrap()
        {
            Value::List(vals) => {
                assert_eq!(vals.len(), 1);
                assert_eq!(vals[0], Value::String("".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_replace_no_match() {
        // Replace when pattern not found
        assert_eq!(
            replace(&[
                Value::String("hello world".to_string()),
                Value::String("foo".to_string()),
                Value::String("bar".to_string()),
            ])
            .unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_wordcount_single_word() {
        assert_eq!(
            wordcount(&[Value::String("hello".to_string())]).unwrap(),
            Value::Int(1)
        );
    }

    #[test]
    fn test_char_len_empty() {
        assert_eq!(
            char_len(&[Value::String("".to_string())]).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_substr_partial() {
        // Substr that extends beyond string length
        assert_eq!(
            substr(&[
                Value::String("hello".to_string()),
                Value::Int(3),
                Value::Int(10),
            ])
            .unwrap(),
            Value::String("lo".to_string())
        );
    }

    // Tests moved from src/libs/expr/tests/functions.rs
    #[test]
    fn test_trim_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["  hello  ".to_string()];
        assert_eq!(
            eval_expr("trim(@1)", &row, None).unwrap().to_string(),
            "hello"
        );
    }

    #[test]
    fn test_upper_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["hello".to_string()];
        assert_eq!(
            eval_expr("upper(@1)", &row, None).unwrap().to_string(),
            "HELLO"
        );
    }

    #[test]
    fn test_lower_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["HELLO".to_string()];
        assert_eq!(
            eval_expr("lower(@1)", &row, None).unwrap().to_string(),
            "hello"
        );
    }

    #[test]
    fn test_len_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["hello".to_string()];
        assert_eq!(eval_expr("len(@1)", &row, None).unwrap().to_string(), "5");
    }
}
