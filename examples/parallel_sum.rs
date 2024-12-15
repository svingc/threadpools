// SPDX-License-Identifier: MIT

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use threadpools::standardpool::{StandardPool, Task};

/// Task that sums a portion of an array in parallel
struct SumTask {
    data: Arc<Vec<u32>>, // Input array to sum
    start: usize,        // Start index of this task's portion
    end: usize,          // End index of this task's portion
    result: AtomicU64,   // Partial sum result for this task
}

impl Task for SumTask {
    fn execute(&self) {
        // Calculate sum for this task's portion of the array
        let sum: u64 = self.data[self.start..self.end]
            .iter()
            .map(|&x| x as u64)
            .sum();
        self.result.fetch_add(sum, Ordering::SeqCst);
    }
}

fn main() {
    let pool = StandardPool::new(4);

    // Initialize test array with all 1's
    let data = Arc::new(vec![1; 1024]);

    let mut tasks: Vec<Arc<SumTask>> = Vec::with_capacity(4);
    for i in 0..4 {
        let chunk_size = data.len() / 4;
        let start = i * chunk_size;
        let end = start + chunk_size;

        let task = Arc::new(SumTask {
            data: data.clone(),
            start,
            end,
            result: AtomicU64::new(0),
        });

        pool.add_task(task.clone());
        tasks.push(task.clone());
    }

    pool.barrier();

    // Aggregate partial sums from all tasks
    let mut total_sum: u64 = 0;
    for task in tasks.drain(..) {
        total_sum += task.result.load(Ordering::SeqCst) as u64;
    }

    println!("Sum: {}", total_sum);
}
