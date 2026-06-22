pub mod channel;
pub mod runtime;
pub mod task;

use runtime::{registry, spawn_task, tokio_rt};
use helper::Value;

pub fn async_run(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_RUN expects 1 argument (task_name)".into());
    }
    let name = string_arg(&args[0], "task_name")?;

    let task_id = spawn_task(async move { Ok(Value::String(name)) });
    Ok(Value::String(task_id))
}

pub fn async_spawn(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("ASYNC_SPAWN expects 2 arguments (task_name, label)".into());
    }
    let name = string_arg(&args[0], "task_name")?;
    let label = string_arg(&args[1], "label")?;

    let task_id = spawn_task(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(Value::String(format!("{}: {}", name, label)))
    });

    Ok(Value::String(task_id))
}

pub fn async_await(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_AWAIT expects 1 argument (task_id)".into());
    }
    let task_id = string_arg(&args[0], "task_id")?;
    let rt = tokio_rt();

    rt.block_on(async move {
        let reg_arc = registry();
        let handle = {
            let reg = reg_arc.lock().await;
            reg.tasks
                .get(&task_id)
                .cloned()
                .ok_or_else(|| format!("Task '{}' not found", task_id))?
        };
        handle.join().await
    })
}

pub fn async_await_all(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_AWAIT_ALL expects 1 argument (array of task_ids)".into());
    }

    let ids = match &args[0] {
        Value::Array(arr) => arr
            .iter()
            .map(|v| string_arg(v, "task_id"))
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("ASYNC_AWAIT_ALL expects an array".into()),
    };

    let rt = tokio_rt();

    rt.block_on(async move {
        let handles: Vec<_> = {
            let reg_arc = registry();
            let reg = reg_arc.lock().await;
            ids.iter()
                .map(|id| {
                    reg.tasks
                        .get(id)
                        .cloned()
                        .ok_or_else(|| format!("Task '{}' not found", id))
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let results = futures::future::join_all(handles.iter().map(|h| h.join())).await;

        Ok(Value::Array(
            results
                .into_iter()
                .map(|r| r.unwrap_or_else(|e| Value::String(format!("Error: {}", e))))
                .collect(),
        ))
    })
}

pub fn async_parallel(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_PARALLEL expects 1 argument (array of values)".into());
    }

    let tasks = match &args[0] {
        Value::Array(arr) => arr.clone(),
        _ => return Err("ASYNC_PARALLEL expects an array".into()),
    };

    let rt = tokio_rt();

    rt.block_on(async move {
        let spawned: Vec<_> = tasks
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    match v {
                        Value::String(s) => Value::String(format!("Task {}: {}", i, s)),
                        Value::Number(n) => Value::Number(n * 2.0),
                        other => other,
                    }
                })
            })
            .collect();

        let results = futures::future::join_all(spawned).await;

        Ok(Value::Array(
            results
                .into_iter()
                .map(|r| r.unwrap_or(Value::String("Task panicked".into())))
                .collect(),
        ))
    })
}

pub fn async_sleep(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_SLEEP expects 1 argument (milliseconds)".into());
    }
    let ms = match &args[0] {
        Value::Number(n) => *n as u64,
        _ => return Err("ASYNC_SLEEP expects a number".into()),
    };

    tokio_rt().block_on(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
        Ok(Value::Boolean(true))
    })
}

pub fn async_status(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_STATUS expects 1 argument (task_id)".into());
    }
    let task_id = string_arg(&args[0], "task_id")?;
    let rt = tokio_rt();

    rt.block_on(async move {
        let reg_arc = registry();
        let handle = {
            let reg = reg_arc.lock().await;
            reg.tasks
                .get(&task_id)
                .cloned()
                .ok_or_else(|| format!("Task '{}' not found", task_id))?
        };

        Ok(Value::String(handle.status().await.to_string()))
    })
}

pub fn async_cancel(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ASYNC_CANCEL expects 1 argument (task_id)".into());
    }
    let task_id = string_arg(&args[0], "task_id")?;
    let rt = tokio_rt();

    rt.block_on(async move {
        let reg_arc = registry();
        let handle = {
            let reg = reg_arc.lock().await;
            reg.tasks
                .get(&task_id)
                .cloned()
                .ok_or_else(|| format!("Task '{}' not found", task_id))?
        };

        handle.cancel().await;
        Ok(Value::Boolean(true))
    })
}

fn string_arg(v: &Value, name: &str) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(format!("Expected string for '{}'", name)),
    }
}
