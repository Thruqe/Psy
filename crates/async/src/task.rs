use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use helper::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

struct TaskInner {
    pub status: TaskStatus,
    pub result: Option<Result<Value, String>>,
}

/// A cloneable handle to a running task. The `Notify` lets `await`
/// block with zero CPU cost until the task signals completion.
#[derive(Clone)]
pub struct TaskHandle {
    id: String,
    inner: Arc<Mutex<TaskInner>>,
    done: Arc<Notify>,
}

impl TaskHandle {
    pub fn new(id: String) -> Self {
        TaskHandle {
            id,
            inner: Arc::new(Mutex::new(TaskInner {
                status: TaskStatus::Pending,
                result: None,
            })),
            done: Arc::new(Notify::new()),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn set_status(&self, status: TaskStatus) {
        let mut inner = self.inner.lock().await;
        inner.status = status;
    }

    /// Store the result and wake any awaiting callers.
    pub async fn set_result(&self, result: Result<Value, String>) {
        let mut inner = self.inner.lock().await;
        inner.status = match &result {
            Ok(_) => TaskStatus::Completed,
            Err(_) => TaskStatus::Failed,
        };
        inner.result = Some(result);
        drop(inner); // release lock before notifying
        self.done.notify_waiters();
    }

    /// Block (async) until the task completes, then return its value.
    pub async fn join(&self) -> Result<Value, String> {
        loop {
            {
                let inner = self.inner.lock().await;
                if inner.result.is_some() {
                    return inner.result.clone().unwrap();
                }
            }
            // Wait for the done signal, then re-check.
            self.done.notified().await;
        }
    }

    pub async fn status(&self) -> TaskStatus {
        self.inner.lock().await.status.clone()
    }

    pub async fn cancel(&self) {
        let mut inner = self.inner.lock().await;
        inner.status = TaskStatus::Cancelled;
        inner.result = Some(Err("Cancelled".to_string()));
        drop(inner);
        self.done.notify_waiters();
    }
}
