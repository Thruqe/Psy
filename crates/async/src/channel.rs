use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use types::Value;
use uuid::Uuid;

struct ChannelEntry {
    tx: mpsc::Sender<Value>,
    rx: Mutex<mpsc::Receiver<Value>>,
}

static CHANNELS: OnceCell<Arc<Mutex<HashMap<String, Arc<ChannelEntry>>>>> = OnceCell::new();

fn channel_registry() -> Arc<Mutex<HashMap<String, Arc<ChannelEntry>>>> {
    CHANNELS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

pub fn channel_create(_args: &[Value]) -> Result<Value, String> {
    let rt = crate::runtime::tokio_rt();
    let id = Uuid::new_v4().to_string();

    rt.block_on(async {
        let (tx, rx) = mpsc::channel::<Value>(64);
        let entry = Arc::new(ChannelEntry {
            tx,
            rx: Mutex::new(rx),
        });
        let reg = channel_registry();
        let mut reg = reg.lock().await;
        reg.insert(id.clone(), entry);
    });

    Ok(Value::String(id))
}

pub fn channel_send(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("CHANNEL_SEND expects 2 arguments (channel_id, value)".into());
    }

    let id = string_arg(&args[0], "channel_id")?;
    let value = args[1].clone();
    let rt = crate::runtime::tokio_rt();

    rt.block_on(async {
        let registry = channel_registry();
        let reg = registry.lock().await;
        let entry = reg
            .get(&id)
            .ok_or_else(|| format!("Channel '{}' not found", id))?
            .clone();
        drop(reg);

        entry
            .tx
            .send(value)
            .await
            .map_err(|_| format!("Channel '{}' is closed", id))?;

        Ok(Value::Boolean(true))
    })
}

pub fn channel_receive(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("CHANNEL_RECEIVE expects 1 argument (channel_id)".into());
    }

    let id = string_arg(&args[0], "channel_id")?;
    let rt = crate::runtime::tokio_rt();

    rt.block_on(async {
        let registry = channel_registry();
        let reg = registry.lock().await;
        let entry = reg
            .get(&id)
            .ok_or_else(|| format!("Channel '{}' not found", id))?
            .clone();
        drop(reg);

        let mut rx = entry.rx.lock().await;
        match rx.recv().await {
            Some(v) => Ok(v),
            None => Ok(Value::Undefined),
        }
    })
}

pub fn channel_close(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("CHANNEL_CLOSE expects 1 argument (channel_id)".into());
    }

    let id = string_arg(&args[0], "channel_id")?;
    let rt = crate::runtime::tokio_rt();

    rt.block_on(async {
        let registry = channel_registry();
        let mut reg = registry.lock().await;
        reg.remove(&id)
            .ok_or_else(|| format!("Channel '{}' not found", id))?;
        Ok(Value::Boolean(true))
    })
}

fn string_arg(v: &Value, name: &str) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(format!("Expected string for '{}'", name)),
    }
}
