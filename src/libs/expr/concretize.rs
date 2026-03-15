/// Concretization - convert AST (Expr) to optimized ConcreteExpr
use crate::libs::expr::concrete::{CompileContext, ConcreteExpr, ConcretePipeRight};
use crate::libs::expr::functions::global_registry;
use crate::libs::expr::parser::ast::{ColumnRef, Expr, PipeRight};
use crate::libs::expr::runtime::value::Value;

/// Convert an Expr to ConcreteExpr
/// This resolves all names to indices and pre-computes constants
pub fn concretize(
    expr: &Expr,
    ctx: &mut CompileContext,
) -> Result<ConcreteExpr, String> {
    match expr {
        Expr::ColumnRef(col_ref) => match col_ref {
            ColumnRef::Index(idx) => Ok(ConcreteExpr::Column(idx - 1)), // Convert to 0-based
            ColumnRef::Name(name) => {
                // First check if it's a variable (bound by 'as') - variables can shadow columns
                if let Some(var_idx) = ctx.get_var(name) {
                    Ok(ConcreteExpr::Variable(var_idx))
                } else if let Some(idx) = ctx.resolve_column(name) {
                    // If not a variable, try to resolve as column
                    Ok(ConcreteExpr::Column(idx))
                } else {
                    Err(format!("Column or variable '{}' not found", name))
                }
            }
        },

        Expr::Variable(name) => {
            if let Some(idx) = ctx.get_var(name) {
                Ok(ConcreteExpr::Variable(idx))
            } else {
                Err(format!("Variable '{}' not found", name))
            }
        }

        Expr::LambdaParam(name) => {
            if let Some(idx) = ctx.get_lambda_param(name) {
                Ok(ConcreteExpr::LambdaParam(idx))
            } else {
                Err(format!("Lambda parameter '{}' not found", name))
            }
        }

        Expr::Int(n) => Ok(ConcreteExpr::Constant(Value::Int(*n))),
        Expr::Float(f) => Ok(ConcreteExpr::Constant(Value::Float(*f))),
        Expr::String(s) => Ok(ConcreteExpr::Constant(Value::String(s.clone()))),
        Expr::Bool(b) => Ok(ConcreteExpr::Constant(Value::Bool(*b))),
        Expr::Null => Ok(ConcreteExpr::Constant(Value::Null)),

        Expr::List(items) => {
            let concrete_items: Result<Vec<_>, _> =
                items.iter().map(|item| concretize(item, ctx)).collect();
            Ok(ConcreteExpr::Call {
                func: global_registry()
                    .get("list")
                    .ok_or("list function not found")?
                    .func,
                args: concrete_items?,
            })
        }

        Expr::Unary { op, expr } => {
            let concrete_expr = concretize(expr, ctx)?;
            Ok(ConcreteExpr::Unary {
                op: *op,
                expr: Box::new(concrete_expr),
            })
        }

        Expr::Binary { op, left, right } => {
            let concrete_left = concretize(left, ctx)?;
            let concrete_right = concretize(right, ctx)?;
            Ok(ConcreteExpr::Binary {
                op: *op,
                left: Box::new(concrete_left),
                right: Box::new(concrete_right),
            })
        }

        Expr::Call { name, args } => {
            let func_info = global_registry()
                .get(name)
                .ok_or_else(|| format!("Function '{}' not found", name))?;
            let concrete_args: Result<Vec<_>, _> =
                args.iter().map(|arg| concretize(arg, ctx)).collect();
            Ok(ConcreteExpr::Call {
                func: func_info.func,
                args: concrete_args?,
            })
        }

        Expr::MethodCall { object, name, args } => {
            let func_info = global_registry()
                .get(name)
                .ok_or_else(|| format!("Method '{}' not found", name))?;
            let concrete_object = concretize(object, ctx)?;
            let concrete_args: Result<Vec<_>, _> =
                args.iter().map(|arg| concretize(arg, ctx)).collect();
            Ok(ConcreteExpr::MethodCall {
                object: Box::new(concrete_object),
                func: func_info.func,
                args: concrete_args?,
            })
        }

        Expr::Pipe { left, right } => {
            let concrete_left = concretize(left, ctx)?;
            let concrete_right = concretize_pipe_right(right, ctx)?;
            Ok(ConcreteExpr::Pipe {
                left: Box::new(concrete_left),
                right: Box::new(concrete_right),
            })
        }

        Expr::Bind { expr, name } => {
            let var_index = ctx.get_or_create_var(name);
            let concrete_expr = concretize(expr, ctx)?;
            Ok(ConcreteExpr::Bind {
                expr: Box::new(concrete_expr),
                var_index,
            })
        }

        Expr::Lambda { params, body } => {
            ctx.push_lambda_scope(params.clone());
            let concrete_body = concretize(body, ctx)?;
            ctx.pop_lambda_scope();
            Ok(ConcreteExpr::Lambda {
                params: params.clone(),
                body: Box::new(concrete_body),
            })
        }

        Expr::Block(exprs) => {
            let concrete_exprs: Result<Vec<_>, _> =
                exprs.iter().map(|e| concretize(e, ctx)).collect();
            Ok(ConcreteExpr::Block(concrete_exprs?))
        }
    }
}

/// Convert PipeRight to ConcretePipeRight
fn concretize_pipe_right(
    pipe_right: &PipeRight,
    ctx: &mut CompileContext,
) -> Result<ConcretePipeRight, String> {
    match pipe_right {
        PipeRight::Call { name, args } => {
            let func_info = global_registry()
                .get(name)
                .ok_or_else(|| format!("Function '{}' not found", name))?;
            let concrete_args: Result<Vec<_>, _> =
                args.iter().map(|arg| concretize(arg, ctx)).collect();
            Ok(ConcretePipeRight::Call {
                func: func_info.func,
                args: concrete_args?,
            })
        }

        PipeRight::CallWithPlaceholder { name, args } => {
            let func_info = global_registry()
                .get(name)
                .ok_or_else(|| format!("Function '{}' not found", name))?;
            // Find placeholder index
            let placeholder_index = args
                .iter()
                .position(|arg| matches!(arg, Expr::Variable(name) if name == "_"))
                .ok_or("Placeholder _ not found in pipe right side")?;
            // Convert non-placeholder args
            let concrete_args: Result<Vec<_>, _> = args
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != placeholder_index)
                .map(|(_, arg)| concretize(arg, ctx))
                .collect();
            Ok(ConcretePipeRight::CallWithPlaceholder {
                func: func_info.func,
                placeholder_index,
                args: concrete_args?,
            })
        }
    }
}

/// Try to evaluate a constant expression at compile time
pub fn try_fold_constant(expr: &ConcreteExpr) -> Option<Value> {
    match expr {
        ConcreteExpr::Constant(v) => Some(v.clone()),
        // Add more constant folding cases here
        _ => None,
    }
}
