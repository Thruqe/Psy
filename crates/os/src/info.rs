use std::env;
use sysinfo::System;
use helper::Value;

/// OS_PLATFORM() → "linux" | "macos" | "windows" | "unknown"
pub fn os_platform(_args: &[Value]) -> Result<Value, String> {
    let platform = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };
    Ok(Value::String(platform.to_string()))
}

/// OS_CWD() → current working directory string
pub fn os_cwd(_args: &[Value]) -> Result<Value, String> {
    match env::current_dir() {
        Ok(path) => Ok(Value::String(path.to_string_lossy().to_string())),
        Err(e) => Err(format!("OS_CWD failed: {}", e)),
    }
}

/// OS_HOSTNAME() → hostname string
pub fn os_hostname(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::String(
        System::host_name().unwrap_or_else(|| "unknown".to_string()),
    ))
}

/// OS_ARGS() → array of CLI argument strings passed after the script name
pub fn os_args(_args: &[Value]) -> Result<Value, String> {
    // argv[0] = psy binary, argv[1] = script path, argv[2..] = user args
    let args: Vec<Value> = env::args().skip(2).map(Value::String).collect();
    Ok(Value::Array(args))
}

/// OS_CPU() → [model, physical_cores, logical_cores, usage_percent]
pub fn os_cpu(_args: &[Value]) -> Result<Value, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let model = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let physical_cores = System::physical_core_count().unwrap_or(0) as f64;
    let logical_cores = sys.cpus().len() as f64;

    // Average CPU usage across all logical cores
    let usage = if sys.cpus().is_empty() {
        0.0
    } else {
        let total: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum();
        (total / sys.cpus().len() as f32) as f64
    };

    Ok(Value::Array(vec![
        Value::String(model),
        Value::Number(physical_cores),
        Value::Number(logical_cores),
        Value::Number(usage),
    ]))
}

/// OS_RAM() → [total_mb, used_mb, free_mb, usage_percent]
pub fn os_ram(_args: &[Value]) -> Result<Value, String> {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total = sys.total_memory() as f64 / 1024.0 / 1024.0;
    let used = sys.used_memory() as f64 / 1024.0 / 1024.0;
    let free = sys.free_memory() as f64 / 1024.0 / 1024.0;
    let usage_pct = if total > 0.0 {
        (used / total) * 100.0
    } else {
        0.0
    };

    Ok(Value::Array(vec![
        Value::Number(total),
        Value::Number(used),
        Value::Number(free),
        Value::Number(usage_pct),
    ]))
}
