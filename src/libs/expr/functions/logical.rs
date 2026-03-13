use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn if_fn(args: &[Value]) -> Result<Value, EvalError> {
    let condition = args[0].as_bool();
    if condition {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

pub fn default_fn(args: &[Value]) -> Result<Value, EvalError> {
    if args[0].is_null() || args[0].as_bool() == false {
        Ok(args[1].clone())
    } else {
        Ok(args[0].clone())
    }
}
