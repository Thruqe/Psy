use hmac::{Hmac, Mac};
use psy_types::Value;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn hmac_generate(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("HMAC_GENERATE expects 2 arguments (text, key)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err("HMAC_GENERATE expects a string as first argument".to_string()),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err("HMAC_GENERATE expects a string key as second argument".to_string()),
    };

    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).map_err(|e| format!("Invalid key: {}", e))?;

    mac.update(text.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    Ok(Value::String(hex::encode(code_bytes)))
}

pub fn hmac_verify(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("HMAC_VERIFY expects 3 arguments (text, hmac, key)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err("HMAC_VERIFY expects a string as first argument".to_string()),
    };

    let hmac_hex = match &args[1] {
        Value::String(s) => s,
        _ => return Err("HMAC_VERIFY expects a string HMAC as second argument".to_string()),
    };

    let key = match &args[2] {
        Value::String(s) => s,
        _ => return Err("HMAC_VERIFY expects a string key as third argument".to_string()),
    };

    let expected_bytes =
        hex::decode(hmac_hex).map_err(|e| format!("Invalid HMAC format: {}", e))?;

    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).map_err(|e| format!("Invalid key: {}", e))?;

    mac.update(text.as_bytes());

    match mac.verify_slice(&expected_bytes) {
        Ok(()) => Ok(Value::Boolean(true)),
        Err(_) => Ok(Value::Boolean(false)),
    }
}
