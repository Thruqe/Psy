use types::Value;
use std::fs;
use std::path::Path;

pub fn delete(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("DELETE expects 1 argument (path)".to_string());
    }

    let path = match &args[0] {
        Value::String(p) => p,
        _ => return Err("DELETE expects a string path argument".to_string()),
    };

    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(format!("Path '{}' does not exist", path));
    }

    let result = if path_obj.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    match result {
        Ok(()) => Ok(Value::Undefined),
        Err(e) => Err(format!("Failed to delete '{}': {}", path, e)),
    }
}
