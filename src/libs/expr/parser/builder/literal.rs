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
