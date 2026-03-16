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

    // Check for global variable: @__xxx
    // Note: Keep the '__' prefix in the name for consistency with 'as @__xxx' binding
    if inner.starts_with("__") {
        if inner.len() == 2 {
            return Err(ParseError::InvalidColumnIndex(s.to_string()));
        }
        return Ok(Expr::GlobalVar(inner.to_string()));
    }

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

#[cfg(test)]
mod tests {
    use super::super::super::ast::{ColumnRef, Expr};
    use super::super::super::parse;

    #[test]
    fn test_parse_column_ref_index() {
        let expr = parse("@1").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Index(1))));
    }

    #[test]
    fn test_parse_column_ref_whole_row() {
        // @0 refers to the whole row (all columns joined with tabs)
        let expr = parse("@0").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::WholeRow)));
    }

    #[test]
    fn test_parse_column_ref_name() {
        let expr = parse("@price").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Name(s)) if s == "price"));
    }

    #[test]
    fn test_parse_variable_ref() {
        // @name is parsed as ColumnRef::Name, not Variable
        // Variable resolution happens at runtime based on context
        let expr = parse("@total").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Name(s)) if s == "total"));
    }

    #[test]
    fn test_parse_column_ref_edge_cases() {
        // Column names with underscores
        let expr = parse("@user_name").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Name(s)) if s == "user_name"));

        // Column names with numbers
        let expr = parse("@col123").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Name(s)) if s == "col123"));

        // Large column index
        let expr = parse("@999").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Index(999))));
    }

    #[test]
    fn test_parse_column_index_negative() {
        // Negative column indices should fail
        assert!(parse("@-1").is_err());
    }

    #[test]
    fn test_parse_invalid_column_name() {
        // Column names must start with a letter or underscore
        assert!(parse("@123abc").is_ok()); // This is parsed as index 123, not name
    }

    #[test]
    fn test_parse_method_call() {
        // @name.trim() should parse as MethodCall { object: @name, name: "trim", args: [] }
        let expr = parse("@name.trim()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(
                    matches!(*object, Expr::ColumnRef(ColumnRef::Name(s)) if s == "name")
                );
                assert_eq!(name, "trim");
                assert!(args.is_empty());
            }
            _ => panic!("Expected MethodCall expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_method_call_with_args() {
        // @name.substr(0, 5) should parse as MethodCall { object: @name, name: "substr", args: [0, 5] }
        let expr = parse("@name.substr(0, 5)").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(
                    matches!(*object, Expr::ColumnRef(ColumnRef::Name(s)) if s == "name")
                );
                assert_eq!(name, "substr");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expr::Int(0)));
                assert!(matches!(args[1], Expr::Int(5)));
            }
            _ => panic!("Expected MethodCall expression with args"),
        }
    }

    #[test]
    fn test_parse_method_chain() {
        // @name.trim().upper() should parse as MethodCall { object: MethodCall { object: @name, name: "trim" }, name: "upper" }
        let expr = parse("@name.trim().upper()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "upper");
                assert!(args.is_empty());
                // Check inner method call
                match *object {
                    Expr::MethodCall {
                        object: inner_obj,
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_name, "trim");
                        assert!(inner_args.is_empty());
                        assert!(
                            matches!(*inner_obj, Expr::ColumnRef(ColumnRef::Name(s)) if s == "name")
                        );
                    }
                    _ => panic!("Expected nested MethodCall for trim"),
                }
            }
            _ => panic!("Expected MethodCall expression for method chain"),
        }
    }
}
