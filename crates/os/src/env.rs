use std::env;
use helper::Value;

/// OS_ENV_GET(key) → string value | Undefined
pub fn os_env_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("OS_ENV_GET expects 1 argument (key)".into());
    }
    let key = string_arg(&args[0], "key")?;
    match env::var(&key) {
        Ok(val) => Ok(Value::String(val)),
        Err(_) => Ok(Value::Undefined),
    }
}

/// OS_ENV_SET(key, value) → true
pub fn os_env_set(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("OS_ENV_SET expects 2 arguments (key, value)".into());
    }
    let key = string_arg(&args[0], "key")?;
    let val = string_arg(&args[1], "value")?;
    unsafe { env::set_var(&key, &val) };
    Ok(Value::Boolean(true))
}

fn string_arg(v: &Value, name: &str) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(format!("Expected string for '{}'", name)),
    }
}
