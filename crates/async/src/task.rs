use std::sync::{Arc, Mutex};
use types::Value;

#[derive(Clone)]
pub struct TaskHandle {
    id: String,
    result: Arc<Mutex<Option<Result<Value, String>>>>,
}

impl TaskHandle {
    pub fn new(id: String) -> Self {
        TaskHandle {
            id,
            result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_result(&self, result: Result<Value, String>) {
        let mut res = self.result.lock().unwrap();
        *res = Some(result);
    }

    pub fn get_result(&self) -> Option<Result<Value, String>> {
        let res = self.result.lock().unwrap();
        res.clone()
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
