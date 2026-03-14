use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn print(args: &[Value]) -> Result<Value, EvalError> {
    let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", parts.join(" "));
    Ok(args.last().unwrap().clone())
}

pub fn eprint(args: &[Value]) -> Result<Value, EvalError> {
    let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    eprintln!("{}", parts.join(" "));
    Ok(args.last().unwrap().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print() {
        let result = print(&[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_print_multiple_args() {
        let result = print(&[
            Value::String("a".to_string()),
            Value::Int(1),
            Value::Bool(true),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eprint() {
        let result = eprint(&[Value::Int(42)]);
        assert_eq!(result.unwrap(), Value::Int(42));
    }
}
