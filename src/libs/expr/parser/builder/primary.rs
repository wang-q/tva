use super::{Expr, ParseError};
use pest::iterators::Pair;

pub fn build_primary(pair: Pair<super::super::Rule>) -> Result<Expr, ParseError> {
    for inner in pair.into_inner() {
        return super::build_expr(inner);
    }
    Err(ParseError::EmptyExpression)
}

pub fn build_column_ref(s: &str) -> Result<Expr, ParseError> {
    use super::ColumnRef;

    let inner = &s[1..]; // Remove '@'

    // Check if it's a quoted column name (starts with " or ')
    if inner.starts_with('"') || inner.starts_with('\'') {
        // Extract the content between quotes
        let quote_char = inner.chars().next().unwrap();
        let end = inner.rfind(quote_char).unwrap_or(inner.len());
        let name = &inner[1..end];
        return Ok(Expr::ColumnRef(ColumnRef::Name(name.to_string())));
    }

    // Check for negative number (invalid column index)
    if inner.starts_with('-') {
        return Err(ParseError::InvalidColumnIndex(inner.to_string()));
    }

    if inner
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        // Parse only the numeric part (stop at first non-digit)
        let numeric_part: String =
            inner.chars().take_while(|c| c.is_ascii_digit()).collect();
        let idx: usize = numeric_part
            .parse()
            .map_err(|_| ParseError::InvalidColumnIndex(numeric_part.clone()))?;
        if idx == 0 {
            Ok(Expr::ColumnRef(ColumnRef::WholeRow))
        } else {
            Ok(Expr::ColumnRef(ColumnRef::Index(idx)))
        }
    } else {
        Ok(Expr::ColumnRef(ColumnRef::Name(inner.to_string())))
    }
}

pub fn build_variable_ref(s: &str) -> Result<Expr, ParseError> {
    let name = extract_variable_name(s);
    Ok(Expr::Variable(name))
}

fn extract_variable_name(s: &str) -> String {
    s[1..].to_string() // Remove '@' prefix
}
