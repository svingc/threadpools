// SPDX-License-Identifier: MIT

use crate::pools::standardpool::JobScheduler;
use crate::support::tasks;
use std::sync::atomic::Ordering::SeqCst;
use std::time::Instant;

/// Benchmarks the performance of parallel array sum computation using the
/// standard thread pool.
///
/// # Arguments
/// * `thread_count` - Number of threads in the pool
/// * `task_count` - Number of tasks to split the work into
/// * `array_size` - Size of the array to sum
///
/// # Returns
/// Duration of the computation in seconds
fn benchmark_subslice_sum(
    thread_count: usize,
    task_count: usize,
    array_size: usize,
) -> f64 {
    // Initialize thread pool scheduler.
    let scheduler = JobScheduler::new(thread_count);

    // Create test array filled with 1's.
    let input_array = vec![1u64; array_size];

    // Start timing.
    let start_time = Instant::now();

    // Create tasks that each sum a portion of the array.
    let mut sum_tasks = crate::support::tasks::get_subslice_sum_tasks(
        input_array.clone(),
        task_count,
    );

    // Submit all tasks to scheduler.
    for task in sum_tasks {
        scheduler.add_task(task.clone());
    }

    // Wait for all tasks to complete
    scheduler.barrier();

    let elapsed_time = start_time.elapsed();

    // Aggregate results from all tasks.
    let mut final_sum = 0u64;
    for task in sum_tasks.drain(..) {
        final_sum += task.subslice_sum.load(SeqCst);
    }

    // Verify result.
    assert_eq!(final_sum as usize, array_size);

    elapsed_time.as_secs_f64()
}

fn main() {
    println!("Running benchmarks...\n");

    // Benchmark 1: Different thread counts with fixed task count and array size
    println!(
        "Benchmark 1: Scaling thread count (task count and array size: {}) ",
        2_u32.pow(17)
    );
    let task_count = 2_u32.pow(17) as usize;
    let array_size = task_count;
    for threads in [4, 8, 16] {
        let time = benchmark_subslice_sum(
            threads,
            2_u32.pow(17) as usize,
            2_u32.pow(17) as usize,
        );
        println!(
            "{:2} threads, {:7} tasks, {:7} array size: {:.6} seconds",
            threads, task_count, array_size, time
        );
    }
    println!();

    // Benchmark 2: Different array sizes with fixed thread and task count
    println!("Benchmark 2: Scaling array size (4 threads, 4 tasks)");
    for size in [100_000, 1_000_000, 10_000_000] {
        let time = benchmark_subslice_sum(4, 4, size);
        println!("{:9} elements (4 threads/tasks): {:.6} seconds", size, time);
    }
    println!();

    // Benchmark 3: Large arrays with different thread/task counts
    println!(
        "Benchmark 3: Large arrays (10,000,000 elements, equal threads/tasks)"
    );
    for threads in [1, 2, 4, 8, 16] {
        let time = benchmark_subslice_sum(threads, threads, 10_000_000);
        println!("{:2} threads/tasks: {:.6} seconds", threads, time);
    }
}
