use psy_types::Value;
use std::time::UNIX_EPOCH;

pub fn format_time(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("FORMATTIME expects 1 argument (timestamp)".to_string());
    }

    let timestamp = match &args[0] {
        Value::Number(n) => {
            if *n < 0.0 {
                return Err("FORMATTIME expects a non-negative timestamp".to_string());
            }
            *n
        }
        _ => return Err("FORMATTIME expects a number argument (timestamp)".to_string()),
    };

    let duration = std::time::Duration::from_secs_f64(timestamp);
    match UNIX_EPOCH.checked_add(duration) {
        Some(_) => {
            let total_seconds = timestamp as u64;
            let days = total_seconds / 86400;
            let hours = (total_seconds % 86400) / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;

            let formatted = format!(
                "{}d {}h {}m {}s (epoch: {})",
                days, hours, minutes, seconds, timestamp
            );

            Ok(Value::String(formatted))
        }
        None => Err("Invalid timestamp value".to_string()),
    }
}
