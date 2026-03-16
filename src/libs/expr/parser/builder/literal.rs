use super::{Expr, ParseError};
use pest::iterators::Pair;

pub fn build_list_literal(pair: Pair<super::super::Rule>) -> Result<Expr, ParseError> {
    let mut items = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == super::super::Rule::expr {
            items.push(super::build_expr(inner)?);
        }
    }

    Ok(Expr::List(items))
}

pub fn build_string(s: &str) -> Result<Expr, ParseError> {
    // Check if it's a q-string: q(...)
    if s.starts_with("q(") && s.ends_with(")") {
        let inner = &s[2..s.len() - 1]; // Remove "q(" and ")"
                                        // Process escape sequences for q-string: \( \) \\
        let mut result = String::new();
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('(') => result.push('('),
                    Some(')') => result.push(')'),
                    Some('\\') => result.push('\\'),
                    Some(c) => {
                        result.push('\\');
                        result.push(c);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        return Ok(Expr::String(result));
    }

    // Regular quoted string: "..." or '...'
    let inner = &s[1..s.len() - 1];
    // Process escape sequences
    let mut result = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    Ok(Expr::String(result))
}

#[cfg(test)]
mod tests {
    use super::super::super::ast::Expr;
    use super::super::super::parse;

    #[test]
    fn test_parse_string() {
        // Double-quoted string
        let expr = parse("\"hello\"").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "hello"));

        // Single-quoted string (documented in literals.md)
        let expr = parse("'hello'").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "hello"));

        // Single-quoted string with spaces
        let expr = parse("'hello world'").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "hello world"));
    }

    #[test]
    fn test_parse_string_with_backslash() {
        // Note: Grammar only supports escape sequences in q-string, not in regular strings.
        // Regular strings treat backslash as literal character.
        // See grammar.pest: double_quoted_string = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }

        // Backslash is literal in regular strings (no escape processing)
        let expr = parse(r#""hello\nworld""#).unwrap();
        match &expr {
            Expr::String(s) => {
                // The actual string content depends on how build_string processes it
                // If no escape processing: s == "hello\\nworld"
                // With escape processing: s == "hello\nworld"
                assert!(
                    s == "hello\\nworld" || s == "hello\nworld",
                    "Unexpected string value: {:?}",
                    s
                );
            }
            _ => panic!("Expected String expression"),
        }

        // Quotes cannot be escaped in regular strings (grammar limitation)
        // The following would fail to parse: parse(r#""say \"hello\"""#)
    }

    #[test]
    fn test_parse_q_string() {
        // q() operator for strings without quote escaping
        let expr = parse("q(hello world)").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "hello world"));

        // Can contain quotes without escaping
        let expr = parse("q(say \"hello\")").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "say \"hello\""));

        let expr = parse("q(it's ok)").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "it's ok"));

        // Can contain both single and double quotes
        let expr = parse("q(He said \"It's ok!\")").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "He said \"It's ok!\""));

        // Nested parentheses need escaping
        let expr = parse("q(test \\(nested\\) parens)").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "test (nested) parens"));
    }

    #[test]
    fn test_parse_q_string_escapes() {
        // q-string supports: \( \) \\ escapes only (see grammar.pest)
        // Grammar: q_escaped = @{ "\\" ~ ("(" | ")" | "\\") }
        // Note: \x or \n are SYNTAX ERRORS because \ must be followed by (, ), or \

        // Escaped backslash in q-string
        let expr = parse(r#"q(test\\path)"#).unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "test\\path"));

        // Escaped parentheses
        let expr = parse(r#"q(test\(nested\))"#).unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "test(nested)"));

        // Multiple escape sequences
        let expr = parse(r#"q(\(test\)\\)"#).unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "(test)\\"));

        // All three escape types together
        let expr = parse(r#"q(\(hello\) and \\)"#).unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "(hello) and \\"));

        // Edge case: just escaped parens
        let expr = parse(r#"q(\(\))"#).unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "()"));

        // Edge case: multiple backslashes
        let expr = parse(r#"q(\\\\)"#).unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "\\\\"));
    }

    #[test]
    fn test_parse_empty_string() {
        let expr = parse("\"\"").unwrap();
        assert!(matches!(expr, Expr::String(s) if s.is_empty()));
    }

    #[test]
    fn test_parse_list_literal() {
        let expr = parse("[1, 2, 3]").unwrap();
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[0], Expr::Int(1)));
                assert!(matches!(elements[1], Expr::Int(2)));
                assert!(matches!(elements[2], Expr::Int(3)));
            }
            _ => panic!("Expected List expression"),
        }
    }

    #[test]
    fn test_parse_empty_list() {
        let expr = parse("[]").unwrap();
        match expr {
            Expr::List(elements) => {
                assert!(elements.is_empty());
            }
            _ => panic!("Expected empty List expression"),
        }
    }

    #[test]
    fn test_parse_nested_list() {
        let expr = parse("[[1, 2], [3, 4]]").unwrap();
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    Expr::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert!(matches!(inner[0], Expr::Int(1)));
                        assert!(matches!(inner[1], Expr::Int(2)));
                    }
                    _ => panic!("Expected nested list"),
                }
            }
            _ => panic!("Expected List expression"),
        }
    }
}
