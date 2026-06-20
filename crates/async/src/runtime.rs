use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;
use tokio::sync::Mutex;

use crate::task::{TaskHandle, TaskStatus};

/// The single shared Tokio runtime for the entire async module.
/// Using `once_cell` gives us safe lazy initialization without OnceLock's
/// const-fn restrictions.
static TOKIO_RT: OnceCell<TokioRuntime> = OnceCell::new();

pub fn tokio_rt() -> &'static TokioRuntime {
    TOKIO_RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime")
    })
}

/// Central registry that maps task IDs → live TaskHandles.
pub struct AsyncRegistry {
    pub tasks: HashMap<String, TaskHandle>,
    pub counter: u64,
}

impl AsyncRegistry {
    pub fn new() -> Self {
        AsyncRegistry {
            tasks: HashMap::new(),
            counter: 0,
        }
    }

    pub fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("task_{}", self.counter)
    }
}

static REGISTRY: OnceCell<Arc<Mutex<AsyncRegistry>>> = OnceCell::new();

pub fn registry() -> Arc<Mutex<AsyncRegistry>> {
    REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(AsyncRegistry::new())))
        .clone()
}

/// Spawn a future onto the shared runtime and register its handle.
/// Returns the task ID.
pub fn spawn_task<F>(future: F) -> String
where
    F: std::future::Future<Output = Result<types::Value, String>> + Send + 'static,
{
    let rt = tokio_rt();
    let reg = registry();

    // Allocate an ID synchronously by blocking briefly on the registry lock.
    let task_id = rt.block_on(async {
        let mut reg = reg.lock().await;
        reg.next_id()
    });

    let handle = TaskHandle::new(task_id.clone());
    let handle_clone = handle.clone();

    // The actual async work runs inside the Tokio runtime.
    rt.spawn(async move {
        handle_clone.set_status(TaskStatus::Running).await;
        let result = future.await;
        handle_clone.set_result(result).await;
    });

    // Register the handle.
    let task_id_ret = task_id.clone();
    rt.block_on(async move {
        let mut reg = reg.lock().await;
        reg.tasks.insert(task_id, handle);
    });

    task_id_ret
}
