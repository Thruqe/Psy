use psy_types::Value;
use std::fs;

pub fn read_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("READFILE expects 1 argument (path)".to_string());
    }

    match &args[0] {
        Value::String(path) => match fs::read_to_string(path) {
            Ok(content) => Ok(Value::String(content)),
            Err(e) => Err(format!("Failed to read file '{}': {}", path, e)),
        },
        _ => Err("READFILE expects a string path argument".to_string()),
    }
}
