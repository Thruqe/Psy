use sha2::{Digest, Sha256};
use helper::Value;

pub fn hash(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("HASH expects 1 argument (text)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err("HASH expects a string argument".to_string()),
    };

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();

    Ok(Value::String(format!("{:x}", result)))
}
