mod bridge;

use types::Value;

pub fn json_parse(args: &[Value]) -> Result<Value, String> {
    let json_str = match args.first() {
        Some(Value::String(s)) => s,
        Some(_) => return Err("JSON_PARSE expects a string argument".to_string()),
        None => return Err("JSON_PARSE expects 1 argument (json string)".to_string()),
    };

    let serde_val: serde_json::Value =
        serde_json::from_str(json_str.trim()).map_err(|e| format!("JSON_PARSE error: {}", e))?;

    Ok(bridge::from_serde(serde_val))
}

pub fn json_stringify(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(v) => {
            let serde_val = bridge::to_serde(v);
            let s = serde_json::to_string(&serde_val)
                .map_err(|e| format!("JSON_STRINGIFY error: {}", e))?;
            Ok(Value::String(s))
        }
        None => Err("JSON_STRINGIFY expects 1 argument (value)".to_string()),
    }
}

pub fn json_get(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("JSON_GET expects 2 arguments (json_string, key)".to_string());
    }

    let json_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err("JSON_GET expects a string as first argument".to_string()),
    };

    let serde_val: serde_json::Value = serde_json::from_str(json_str.trim())
        .map_err(|e| format!("JSON_GET parse error: {}", e))?;

    let result = match &args[1] {
        Value::String(key) => serde_val
            .get(key.as_str())
            .cloned()
            .ok_or_else(|| format!("Key '{}' not found", key))?,

        Value::Number(n) => {
            let idx = *n as usize;
            serde_val
                .get(idx)
                .cloned()
                .ok_or_else(|| format!("Index {} out of bounds", idx))?
        }

        _ => return Err("JSON_GET key must be a string or number".to_string()),
    };

    Ok(bridge::from_serde(result))
}

pub fn json_set(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("JSON_SET expects 3 arguments (json_string, key, value)".to_string());
    }

    let json_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err("JSON_SET expects a string as first argument".to_string()),
    };

    let key = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("JSON_SET key must be a string".to_string()),
    };

    let mut serde_val: serde_json::Value = serde_json::from_str(json_str.trim())
        .map_err(|e| format!("JSON_SET parse error: {}", e))?;

    match &mut serde_val {
        serde_json::Value::Object(map) => {
            map.insert(key, bridge::to_serde(&args[2]));
        }
        _ => return Err("JSON_SET requires a JSON object as first argument".to_string()),
    }

    let s = serde_json::to_string(&serde_val)
        .map_err(|e| format!("JSON_SET serialize error: {}", e))?;

    Ok(Value::String(s))
}

pub fn json_keys(args: &[Value]) -> Result<Value, String> {
    let json_str = match args.first() {
        Some(Value::String(s)) => s,
        Some(_) => return Err("JSON_KEYS expects a string argument".to_string()),
        None => return Err("JSON_KEYS expects 1 argument (json_string)".to_string()),
    };

    let serde_val: serde_json::Value = serde_json::from_str(json_str.trim())
        .map_err(|e| format!("JSON_KEYS parse error: {}", e))?;

    match serde_val {
        serde_json::Value::Object(map) => {
            let keys = map.keys().map(|k| Value::String(k.clone())).collect();
            Ok(Value::Array(keys))
        }
        _ => Err("JSON_KEYS requires a JSON object".to_string()),
    }
}
