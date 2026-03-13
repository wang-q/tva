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
