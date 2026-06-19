pub mod channel;
pub mod runtime;
pub mod task;

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::thread;
use types::Value;

// Global async runtime
fn async_runtime() -> &'static Arc<Mutex<AsyncRuntime>> {
    static RUNTIME: OnceLock<Arc<Mutex<AsyncRuntime>>> = OnceLock::new();
    RUNTIME.get_or_init(|| Arc::new(Mutex::new(AsyncRuntime::new())))
}

struct AsyncRuntime {
    tasks: HashMap<String, AsyncTask>,
    task_counter: u64,
    // running field removed since it's never used
}

struct AsyncTask {
    id: String,
    name: String,
    status: TaskStatus,
    result: Option<Value>,
    thread_handle: Option<thread::JoinHandle<Result<Value, String>>>,
}

#[derive(Clone, PartialEq)]
enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl AsyncRuntime {
    fn new() -> Self {
        AsyncRuntime {
            tasks: HashMap::new(),
            task_counter: 0,
        }
    }
}

// Native function: ASYNC_RUN
pub fn async_run(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_RUN expects at least 1 argument (task_name)".to_string());
    }

    let task_name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("ASYNC_RUN expects a string task name".to_string()),
    };

    let runtime = async_runtime();
    let mut runtime = runtime.lock().map_err(|e| format!("Lock error: {}", e))?;

    runtime.task_counter += 1;
    let task_id = format!("task_{}", runtime.task_counter);

    runtime.tasks.insert(
        task_id.clone(),
        AsyncTask {
            id: task_id.clone(),
            name: task_name,
            status: TaskStatus::Pending,
            result: None,
            thread_handle: None,
        },
    );

    Ok(Value::String(task_id))
}

// Native function: ASYNC_SPAWN
pub fn async_spawn(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("ASYNC_SPAWN expects 2 arguments (task_id, function_name)".to_string());
    }

    let task_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("ASYNC_SPAWN expects a string task ID".to_string()),
    };

    let function_name = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("ASYNC_SPAWN expects a string function name".to_string()),
    };

    let runtime = async_runtime();
    let mut runtime = runtime.lock().map_err(|e| format!("Lock error: {}", e))?;

    let task = runtime
        .tasks
        .get_mut(&task_id)
        .ok_or_else(|| format!("Task '{}' not found", task_id))?;

    task.status = TaskStatus::Running;

    // In a real implementation, we'd execute the function
    // For now, we just simulate async execution
    let task_id_clone = task_id.clone();
    let handle = thread::spawn(move || {
        // Simulate work
        thread::sleep(std::time::Duration::from_millis(100));
        Ok(Value::String(format!(
            "Task {} completed: {}",
            task_id_clone, function_name
        )))
    });

    task.thread_handle = Some(handle);

    Ok(Value::Boolean(true))
}

// Native function: ASYNC_AWAIT
pub fn async_await(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_AWAIT expects 1 argument (task_id)".to_string());
    }

    let task_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("ASYNC_AWAIT expects a string task ID".to_string()),
    };

    let runtime = async_runtime();
    let mut runtime = runtime.lock().map_err(|e| format!("Lock error: {}", e))?;

    let task = runtime
        .tasks
        .get_mut(&task_id)
        .ok_or_else(|| format!("Task '{}' not found", task_id))?;

    // Wait for the task to complete
    if let Some(handle) = task.thread_handle.take() {
        match handle.join() {
            Ok(result) => {
                task.status = TaskStatus::Completed;
                task.result = Some(match result {
                    Ok(v) => v,
                    Err(e) => Value::String(e),
                });
                Ok(task.result.clone().unwrap_or(Value::Undefined))
            }
            Err(_) => {
                task.status = TaskStatus::Failed;
                Err("Task panicked".to_string())
            }
        }
    } else {
        Err("Task is not running".to_string())
    }
}

pub fn async_await_all(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_AWAIT_ALL expects at least 1 argument (task_ids array)".to_string());
    }

    let task_ids = match &args[0] {
        Value::Array(arr) => {
            let mut ids = Vec::new();
            for item in arr {
                match item {
                    Value::String(s) => ids.push(s.clone()),
                    _ => {
                        return Err(
                            "ASYNC_AWAIT_ALL expects an array of task ID strings".to_string()
                        );
                    }
                }
            }
            ids
        }
        _ => return Err("ASYNC_AWAIT_ALL expects an array of task IDs".to_string()),
    };

    let mut results = Vec::new();

    for task_id in task_ids {
        match async_await(&[Value::String(task_id)]) {
            Ok(result) => results.push(result),
            Err(e) => results.push(Value::String(format!("Error: {}", e))),
        }
    }

    Ok(Value::Array(results))
}

// Native function: ASYNC_SLEEP
pub fn async_sleep(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_SLEEP expects 1 argument (milliseconds)".to_string());
    }

    let ms = match &args[0] {
        Value::Number(n) => *n as u64,
        _ => return Err("ASYNC_SLEEP expects a number (milliseconds)".to_string()),
    };

    thread::sleep(std::time::Duration::from_millis(ms));
    Ok(Value::Boolean(true))
}

pub fn async_parallel(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_PARALLEL expects at least 1 argument (tasks array)".to_string());
    }

    let tasks = match &args[0] {
        Value::Array(arr) => arr.clone(),
        _ => return Err("ASYNC_PARALLEL expects an array of task configurations".to_string()),
    };

    let mut handles: Vec<thread::JoinHandle<Result<Value, String>>> = Vec::new();

    for (i, task) in tasks.iter().enumerate() {
        let task_clone = task.clone();
        let handle = thread::spawn(move || {
            // Simulate async work
            thread::sleep(std::time::Duration::from_millis(50));
            match task_clone {
                Value::String(s) => Ok(Value::String(format!("Task {} result: {}", i, s))),
                Value::Number(n) => Ok(Value::Number(n * 2.0)),
                _ => Ok(Value::String(format!("Task {} completed", i))),
            }
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(result) => match result {
                Ok(v) => results.push(v),
                Err(e) => results.push(Value::String(format!("Error: {}", e))),
            },
            Err(_) => results.push(Value::String("Task panicked".to_string())),
        }
    }

    Ok(Value::Array(results))
}

// Native function: ASYNC_STATUS
pub fn async_status(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_STATUS expects 1 argument (task_id)".to_string());
    }

    let task_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("ASYNC_STATUS expects a string task ID".to_string()),
    };

    let runtime = async_runtime();
    let runtime = runtime.lock().map_err(|e| format!("Lock error: {}", e))?;

    let task = runtime
        .tasks
        .get(&task_id)
        .ok_or_else(|| format!("Task '{}' not found", task_id))?;

    let status_str = match task.status {
        TaskStatus::Pending => "pending",
        TaskStatus::Running => "running",
        TaskStatus::Completed => "completed",
        TaskStatus::Failed => "failed",
    };

    Ok(Value::String(status_str.to_string()))
}

// Native function: ASYNC_CANCEL
pub fn async_cancel(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_CANCEL expects 1 argument (task_id)".to_string());
    }

    let task_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("ASYNC_CANCEL expects a string task ID".to_string()),
    };

    let runtime = async_runtime();
    let mut runtime = runtime.lock().map_err(|e| format!("Lock error: {}", e))?;

    runtime.tasks.remove(&task_id);
    Ok(Value::Boolean(true))
}
