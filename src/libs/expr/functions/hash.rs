use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use sha2::{Digest, Sha256};

pub fn md5(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let digest = md5::compute(s.as_bytes());
    Ok(Value::String(format!("{:x}", digest)))
}

pub fn sha256(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(Value::String(format!("{:x}", result)))
}

pub fn base64_encode(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    Ok(Value::String(base64::encode(s.as_bytes())))
}

pub fn base64_decode(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    match base64::decode(s.as_bytes()) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(decoded) => Ok(Value::String(decoded)),
            Err(_) => Err(EvalError::TypeError(
                "base64_decode: decoded bytes are not valid UTF-8".to_string()
            )),
        },
        Err(e) => Err(EvalError::TypeError(format!(
            "base64_decode: invalid base64 string: {}",
            e
        ))),
    }
}
