use crate::libs::expr::parser::ast::{Expr, UnaryOp};
use crate::libs::expr::parser::ParseError;
use crate::libs::expr::parser::Rule;
use pest::iterators::Pair;

pub fn build_unary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut ops: Vec<UnaryOp> = Vec::new();
    let mut inner_pairs: Vec<Pair<Rule>> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_not => ops.push(UnaryOp::Not),
            Rule::op_neg => ops.push(UnaryOp::Neg),
            _ => inner_pairs.push(inner),
        }
    }

    // Build the base expression from inner pairs (postfix content)
    // Handle method chain: primary/func_call followed by method_call(s)
    let mut result = if inner_pairs.is_empty() {
        return Err(ParseError::EmptyExpression);
    } else {
        // First pair is the base (primary or func_call)
        let first = &inner_pairs[0];
        let base = match first.as_rule() {
            Rule::func_call => super::build_func_call(first.clone())?,
            Rule::postfix => super::build_postfix(first.clone())?,
            Rule::primary => super::build_primary(first.clone())?,
            _ => super::build_expr(first.clone())?,
        };

        // Remaining pairs are method calls
        let mut obj = base;
        for method_pair in inner_pairs.iter().skip(1) {
            if method_pair.as_rule() == Rule::method_call {
                let (name, args) = super::build_method_call(method_pair.clone())?;
                obj = Expr::MethodCall {
                    object: Box::new(obj),
                    name,
                    args,
                };
            }
        }
        obj
    };

    for op in ops.into_iter().rev() {
        result = Expr::Unary {
            op,
            expr: Box::new(result),
        };
    }

    Ok(result)
}
