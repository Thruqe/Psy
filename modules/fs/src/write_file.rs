use psy_types::Value;
use std::fs;

pub fn write_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("WRITEFILE expects 2 arguments (path, content)".to_string());
    }

    let path = match &args[0] {
        Value::String(p) => p,
        _ => return Err("WRITEFILE expects a string path as first argument".to_string()),
    };

    let content = match &args[1] {
        Value::String(c) => c,
        _ => return Err("WRITEFILE expects a string content as second argument".to_string()),
    };

    match fs::write(path, content) {
        Ok(()) => Ok(Value::Undefined),
        Err(e) => Err(format!("Failed to write file '{}': {}", path, e)),
    }
}
