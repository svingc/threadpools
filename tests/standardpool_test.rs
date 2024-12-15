// SPDX-License-Identifier: MIT

use std::sync::{atomic::Ordering::SeqCst, Arc, Mutex};
use test_utils::tasks::get_subslice_sum_tasks;
use threadpools::standardpool::{StandardPool, Task};

/// Task that increments a shared counter protected by a mutex.
pub struct SharedCounterTask {
    /// Thread-safe counter variable protected by a mutex.
    counter: Arc<Mutex<u32>>,
}

impl SharedCounterTask {
    /// Creates a new SharedCounterTask with the given counter.
    pub fn new(counter: Arc<Mutex<u32>>) -> Self {
        SharedCounterTask { counter }
    }
}

impl Task for SharedCounterTask {
    fn execute(&self) {
        let mut counter_guard = self.counter.lock().unwrap();
        *counter_guard += 1;
    }
}

#[test]
fn test_functionality_shared_variable() {
    // Set up test parameters
    let num_workers = 32;
    let total_tasks = 2048;
    let scheduler = StandardPool::new(num_workers);
    let shared_counter = Arc::new(Mutex::new(0));

    // Run tasks in 8 batches with barriers between each batch
    for _ in 0..8 {
        for _ in 0..total_tasks / 8 {
            scheduler.add_task(Arc::new(SharedCounterTask::new(
                shared_counter.clone(),
            )));
        }
        scheduler.barrier();
    }

    assert_eq!(*shared_counter.lock().unwrap(), total_tasks);
}

#[test]
fn test_functionality_subslices() {
    let num_workers: usize = 32; // 2^5
    let num_tasks: usize = 4096; // 2^12
    let array_size: usize = 4194304; // 2^22
    let num_barriers: usize = 8; // 2^3

    // Initialize the job scheduler with specified number of worker threads.
    let scheduler = StandardPool::new(num_workers);

    // Create array of 1s that will be summed.
    let array = Arc::new(vec![1u32; array_size]);

    // Create tasks that each sum a portion of the array.
    let mut tasks = get_subslice_sum_tasks(array.clone(), num_tasks);

    // Process tasks in chunks with barriers between chunks.
    for task_chunk in tasks.chunks(num_tasks / num_barriers) {
        for task in task_chunk {
            scheduler.add_task(task.clone());
        }
        scheduler.barrier();
    }

    // Sum up results from all tasks.
    let mut total_sum = 0u64;
    for task in tasks.drain(..) {
        total_sum += task.subslice_sum.load(SeqCst);
    }

    // Verify sum equals array size (since array contains all 1s).
    assert_eq!(total_sum, array_size as u64);
}
