use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use base64::Engine;
use sha2::{Digest, Sha256};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        let result = md5(&[Value::String("hello".to_string())]);
        // MD5 of "hello" is 5d41402abc4b2a76b9719d911017c592
        assert_eq!(
            result.unwrap(),
            Value::String("5d41402abc4b2a76b9719d911017c592".to_string())
        );
    }

    #[test]
    fn test_sha256() {
        let result = sha256(&[Value::String("hello".to_string())]);
        // SHA256 of "hello"
        assert_eq!(
            result.unwrap(),
            Value::String(
                "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_base64() {
        let result = base64_encode(&[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("aGVsbG8=".to_string()));

        // Empty string
        let result = base64_encode(&[Value::String("".to_string())]);
        assert_eq!(result.unwrap(), Value::String("".to_string()));
    }

    #[test]
    fn test_unbase64() {
        let result = base64_decode(&[Value::String("aGVsbG8=".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));

        // Round-trip
        let encoded = base64_encode(&[Value::String("test 123".to_string())]).unwrap();
        let result = base64_decode(&[encoded]);
        assert_eq!(result.unwrap(), Value::String("test 123".to_string()));
    }
}

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
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
    ))
}

pub fn base64_decode(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    match base64::engine::general_purpose::STANDARD.decode(s.as_bytes()) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(decoded) => Ok(Value::String(decoded)),
            Err(_) => Err(EvalError::TypeError(
                "base64_decode: decoded bytes are not valid UTF-8".to_string(),
            )),
        },
        Err(e) => Err(EvalError::TypeError(format!(
            "base64_decode: invalid base64 string: {}",
            e
        ))),
    }
}
