// SPDX-License-Identifier: MIT

use crate::slices_utils::ChunkSplitter;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
use threadpools::standardpool::Task;

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

pub struct SubsliceSumTask {
    input_array: Arc<Vec<u32>>,
    slice_start: usize,
    slice_end: usize,
    pub subslice_sum: AtomicU64,
}

impl SubsliceSumTask {
    pub fn new(
        input_array: Arc<Vec<u32>>,
        slice_start: usize,
        slice_end: usize,
    ) -> Self {
        SubsliceSumTask {
            input_array,
            slice_start,
            slice_end,
            subslice_sum: AtomicU64::new(0u64),
        }
    }
}

impl Task for SubsliceSumTask {
    fn execute(&self) {
        let sum: u64 = self.input_array[self.slice_start..self.slice_end]
            .iter()
            .sum::<u32>() as u64;
        self.subslice_sum.store(sum, SeqCst);
    }
}

pub fn get_subslice_sum_tasks(
    input_array: Arc<Vec<u32>>,
    num_tasks: usize,
) -> Vec<Arc<SubsliceSumTask>> {
    let mut sum_tasks: Vec<Arc<SubsliceSumTask>> =
        Vec::with_capacity(num_tasks);
    let chunk_splitter = ChunkSplitter::new(input_array.len(), num_tasks);

    for (start, end) in chunk_splitter {
        sum_tasks.push(Arc::new(SubsliceSumTask::new(
            input_array.clone(),
            start,
            end,
        )));
    }
    sum_tasks
}
