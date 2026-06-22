use std::process::Command;
use helper::Value;

/// OS_EXEC(command) → [exit_code, stdout, stderr]
/// OS_EXEC(command, capture_output) → same, but capture_output=FALSE prints live
pub fn os_exec(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("OS_EXEC expects at least 1 argument (command)".into());
    }

    let command = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("OS_EXEC expects a string command".into()),
    };

    let capture = if args.len() > 1 {
        match &args[1] {
            Value::Boolean(b) => *b,
            _ => true,
        }
    } else {
        true
    };

    // Split command into program + args using shell
    #[cfg(unix)]
    let mut cmd = {
        let mut c = Command::new("sh");
        c.arg("-c").arg(&command);
        c
    };

    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", &command]);
        c
    };

    if capture {
        let output = cmd.output().map_err(|e| format!("OS_EXEC failed: {}", e))?;

        let exit_code = output.status.code().unwrap_or(-1) as f64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(Value::Array(vec![
            Value::Number(exit_code),
            Value::String(stdout),
            Value::String(stderr),
        ]))
    } else {
        // Stream output directly to terminal
        let status = cmd.status().map_err(|e| format!("OS_EXEC failed: {}", e))?;

        let exit_code = status.code().unwrap_or(-1) as f64;
        Ok(Value::Array(vec![
            Value::Number(exit_code),
            Value::String(String::new()),
            Value::String(String::new()),
        ]))
    }
}
