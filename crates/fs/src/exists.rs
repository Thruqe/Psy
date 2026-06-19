use std::path::Path;
use types::Value;

pub fn exists(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("EXISTS expects 1 argument (path)".to_string());
    }

    match &args[0] {
        Value::String(path) => Ok(Value::Boolean(Path::new(path).exists())),
        _ => Err("EXISTS expects a string path argument".to_string()),
    }
}

pub fn is_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("ISFILE expects 1 argument (path)".to_string());
    }

    match &args[0] {
        Value::String(path) => Ok(Value::Boolean(Path::new(path).is_file())),
        _ => Err("ISFILE expects a string path argument".to_string()),
    }
}

pub fn is_dir(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("ISDIR expects 1 argument (path)".to_string());
    }

    match &args[0] {
        Value::String(path) => Ok(Value::Boolean(Path::new(path).is_dir())),
        _ => Err("ISDIR expects a string path argument".to_string()),
    }
}
