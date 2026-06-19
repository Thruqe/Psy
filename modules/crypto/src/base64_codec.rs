use base64::Engine;
use psy_types::Value;

pub fn base64_encode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("BASE64_ENCODE expects 1 argument (text)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err("BASE64_ENCODE expects a string argument".to_string()),
    };

    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(text.as_bytes()),
    ))
}

pub fn base64_decode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("BASE64_DECODE expects 1 argument (encoded_text)".to_string());
    }

    let encoded = match &args[0] {
        Value::String(s) => s,
        _ => return Err("BASE64_DECODE expects a string argument".to_string()),
    };

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    String::from_utf8(decoded)
        .map(Value::String)
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}
