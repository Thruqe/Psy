use std::fs;
use helper::Value;

pub fn list_dir(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("LISTDIR expects 1 argument (path)".to_string());
    }

    let path = match &args[0] {
        Value::String(p) => p,
        _ => return Err("LISTDIR expects a string path argument".to_string()),
    };

    match fs::read_dir(path) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        if let Some(name) = entry.file_name().to_str() {
                            files.push(Value::String(name.to_string()));
                        }
                    }
                    Err(e) => return Err(format!("Error reading directory: {}", e)),
                }
            }
            Ok(Value::Array(files))
        }
        Err(e) => Err(format!("Failed to list directory '{}': {}", path, e)),
    }
}
