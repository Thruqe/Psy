use types::Value;

/// Convert a `serde_json::Value` into a `types::Value`.
/// JSON objects become `Value::Array` of `Value::String("key: <json>")` to
/// stay compatible with the rest of the interpreter, OR you can change this
/// to a future `Value::Object` variant when you add one.
pub fn from_serde(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Undefined,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => Value::Array(arr.into_iter().map(from_serde).collect()),
        serde_json::Value::Object(map) => {
            // Represent the whole object as its JSON string so callers can
            // round-trip it back through JSON_GET / JSON_SET without loss.
            let json = serde_json::Value::Object(map);
            Value::String(json.to_string())
        }
    }
}

/// Convert a `types::Value` into a `serde_json::Value`.
/// Strings that look like JSON are re-parsed so objects survive a round-trip.
pub fn to_serde(v: &Value) -> serde_json::Value {
    match v {
        Value::Undefined => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                if let Ok(v) = serde_json::from_str(trimmed) {
                    return v;
                }
                if let Some(obj) = try_parse_psy_object(trimmed) {
                    return obj;
                }
            }
            serde_json::Value::String(s.clone())
        }
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(to_serde).collect()),
    }
}

fn try_parse_psy_object(s: &str) -> Option<serde_json::Value> {
    // Strip surrounding brackets — accept [] or {}
    let inner = s.trim();
    let inner = if inner.starts_with('[') && inner.ends_with(']') {
        &inner[1..inner.len() - 1]
    } else if inner.starts_with('{') && inner.ends_with('}') {
        &inner[1..inner.len() - 1]
    } else {
        return None;
    };

    let mut map = serde_json::Map::new();

    for pair in inner.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let colon = pair.find(':')?;
        let key = pair[..colon].trim().to_string();
        let val = pair[colon + 1..].trim();

        let json_val = if val == "true" {
            serde_json::Value::Bool(true)
        } else if val == "false" {
            serde_json::Value::Bool(false)
        } else if let Ok(n) = val.parse::<f64>() {
            serde_json::Number::from_f64(n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::String(val.to_string()))
        } else {
            serde_json::Value::String(val.to_string())
        };

        map.insert(key, json_val);
    }

    Some(serde_json::Value::Object(map))
}
