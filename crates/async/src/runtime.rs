use std::sync::{Arc, Mutex};
use std::thread;
use types::Value;

pub struct Runtime {
    _workers: Vec<Worker>,
    task_queue: Arc<Mutex<Vec<Task>>>,
}

struct Worker {
    _id: usize,
    _thread: Option<thread::JoinHandle<()>>,
}

type AsyncFn = Box<dyn FnOnce() -> Result<Value, String> + Send + 'static>;

struct Task {
    _id: String,
    func: AsyncFn,
}

impl Runtime {
    pub fn new(num_workers: usize) -> Self {
        let task_queue: Arc<Mutex<Vec<Task>>> = Arc::new(Mutex::new(Vec::new()));
        let mut workers = Vec::new();

        for id in 0..num_workers {
            let queue = Arc::clone(&task_queue);
            let thread = thread::spawn(move || {
                loop {
                    let task = {
                        let mut queue = queue.lock().unwrap();
                        queue.pop()
                    };

                    match task {
                        Some(task) => {
                            let _result = (task.func)();
                        }
                        None => {
                            thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                }
            });

            workers.push(Worker {
                _id: id,
                _thread: Some(thread),
            });
        }

        Runtime {
            _workers: workers,
            task_queue,
        }
    }

    pub fn spawn<F>(&self, task_id: String, f: F)
    where
        F: FnOnce() -> Result<Value, String> + Send + 'static,
    {
        let task = Task {
            _id: task_id,
            func: Box::new(f),
        };

        let mut queue = self.task_queue.lock().unwrap();
        queue.push(task);
    }
}
