use base64::Engine;
use helper::Value;

pub fn rsa_generate_key(_args: &[Value]) -> Result<Value, String> {
    if !_args.is_empty() {
        return Err("RSA_GENERATE_KEY expects 0 arguments".to_string());
    }

    // Return an array with [public_key, private_key]
    Ok(Value::Array(vec![
        Value::String("PUBLIC_KEY_PLACEHOLDER_12345".to_string()),
        Value::String("PRIVATE_KEY_PLACEHOLDER_67890".to_string()),
    ]))
}

pub fn rsa_encrypt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("RSA_ENCRYPT expects 2 arguments (text, public_key)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err("RSA_ENCRYPT expects a string as first argument".to_string()),
    };

    let _public_key = match &args[1] {
        Value::String(s) => s,
        _ => return Err("RSA_ENCRYPT expects a string key as second argument".to_string()),
    };

    // Simple XOR-based encryption as RSA placeholder
    let key_bytes = b"RSA_PLACEHOLDER_KEY_FOR_DEMO";
    let encrypted: Vec<u8> = text
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(&encrypted),
    ))
}

pub fn rsa_decrypt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("RSA_DECRYPT expects 2 arguments (encrypted_text, private_key)".to_string());
    }

    let encrypted = match &args[0] {
        Value::String(s) => s,
        _ => return Err("RSA_DECRYPT expects a string as first argument".to_string()),
    };

    let _private_key = match &args[1] {
        Value::String(s) => s,
        _ => return Err("RSA_DECRYPT expects a string key as second argument".to_string()),
    };

    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Invalid base64 input: {}", e))?;

    let key_bytes = b"RSA_PLACEHOLDER_KEY_FOR_DEMO";
    let decrypted: Vec<u8> = ciphertext
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    String::from_utf8(decrypted)
        .map(Value::String)
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}
