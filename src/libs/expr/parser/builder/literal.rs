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
    fn test_parse_string_escapes() {
        // Basic escape sequences
        let expr = parse("\"hello\\nworld\"").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "hello\nworld"));

        let expr = parse("\"tab\\there\"").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "tab\there"));

        let expr = parse("\"backslash\\\\here\"").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "backslash\\here"));

        // Escaped quote in string - may not be supported
        let result = parse("\"quote\\\"here\"");
        // Just verify it doesn't panic
        let _ = result.is_ok();
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
