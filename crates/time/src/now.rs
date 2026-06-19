use types::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("NOW expects 0 arguments".to_string());
    }

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(Value::Number(duration.as_secs_f64())),
        Err(e) => Err(format!("Failed to get current time: {}", e)),
    }
}

pub fn now_ms(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("NOWMS expects 0 arguments".to_string());
    }

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(Value::Number(duration.as_millis() as f64)),
        Err(e) => Err(format!("Failed to get current time: {}", e)),
    }
}
