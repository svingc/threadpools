// SPDX-License-Identifier: MIT

extern crate threadpools as root;

use root::pools::standardpool::StandardPool;
use root::support::tasks;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
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
    let scheduler = StandardPool::new(thread_count);

    // Create test array filled with 1's.
    let input_array = Arc::new(vec![1u32; array_size]);

    // Start timing.
    let start_time = Instant::now();

    // Create tasks that each sum a portion of the array.
    let mut sum_tasks =
        tasks::get_subslice_sum_tasks(input_array.clone(), task_count);

    // Submit all tasks to scheduler.
    for task in &sum_tasks {
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

    // Benchmark 1: Testing task queue performance with many small tasks.
    {
        // Set very high task count but small array size per task
        const TASK_COUNT: usize = 2_u32.pow(20) as usize;
        const ARRAY_SIZE: usize = TASK_COUNT / 1000;

        println!("\nBenchmark 1: Testing task queue overhead");

        // Test with varying number of threads
        let thread_counts = [2, 4, 8, 16, 32];
        for &threads in &thread_counts {
            let duration =
                benchmark_subslice_sum(threads, TASK_COUNT, ARRAY_SIZE);

            print!("{{\"threads\": {:2}", threads);
            print!(", \"tasks\": {}", TASK_COUNT);
            print!(", \"array_size\": {}", ARRAY_SIZE);
            print!(", \"time\": {:.6}}}\n", duration);
        }
        println!();
    }

    // Benchmark 2: Different array sizes with fixed thread and task count.
    {
        const THREAD_COUNT: usize = 4;
        const TASK_COUNT: usize = 4;

        println!("\nBenchmark 2: Scaling array size");

        // Test with varying array sizes
        let array_sizes = [1_000_000, 10_000_000, 100_000_000];
        for &size in &array_sizes {
            let duration =
                benchmark_subslice_sum(THREAD_COUNT, TASK_COUNT, size);

            print!("{{\"threads\": {}", THREAD_COUNT);
            print!(", \"tasks\": {}", TASK_COUNT);
            print!(", \"array_size\": {:9}", size);
            print!(", \"time\": {:.6}}}\n", duration);
        }
        println!();
    }

    // Benchmark 3: Testing thread scaling on large arrays
    {
        const ARRAY_SIZE: usize = 50_000_000;

        println!("\nBenchmark 3: Thread scaling with large arrays");

        let thread_counts = [1, 2, 4, 8, 16];
        for &threads in &thread_counts {
            let duration = benchmark_subslice_sum(threads, threads, ARRAY_SIZE);

            print!("{{\"threads\": {:2}", threads);
            print!(", \"tasks\": {:2}", threads);
            print!(", \"array_size\": {}", ARRAY_SIZE);
            print!(", \"time\": {:.6}}}\n", duration);
        }
        println!();
    }
}
