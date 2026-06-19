use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use types::Value;

pub struct Channel {
    queue: Arc<Mutex<VecDeque<Value>>>,
    closed: Arc<Mutex<bool>>,
}

impl Channel {
    pub fn new() -> Self {
        Channel {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    pub fn send(&self, value: Value) -> Result<(), String> {
        let closed = self
            .closed
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        if *closed {
            return Err("Channel is closed".to_string());
        }

        let mut queue = self
            .queue
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        queue.push_back(value);
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<Value>, String> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(queue.pop_front())
    }

    pub fn close(&self) -> Result<(), String> {
        let mut closed = self
            .closed
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *closed = true;
        Ok(())
    }
}

// Native function: CHANNEL_CREATE
pub fn channel_create(args: &[Value]) -> Result<Value, String> {
    let _ = args;
    Ok(Value::String("channel_1".to_string())) // Simplified - would need global channel registry
}

// Native function: CHANNEL_SEND
pub fn channel_send(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("CHANNEL_SEND expects 2 arguments (channel_id, value)".to_string());
    }

    let _channel_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("CHANNEL_SEND expects a string channel ID".to_string()),
    };

    // Simplified - would need actual channel implementation
    Ok(Value::Boolean(true))
}

// Native function: CHANNEL_RECEIVE
pub fn channel_receive(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("CHANNEL_RECEIVE expects 1 argument (channel_id)".to_string());
    }

    let _channel_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("CHANNEL_RECEIVE expects a string channel ID".to_string()),
    };

    // Simplified - would need actual channel implementation
    Ok(Value::String("channel message".to_string()))
}
