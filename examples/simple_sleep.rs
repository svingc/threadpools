use std::sync::Arc;
use std::{thread, time::Duration};
use threadpools::pools::standardpool::{StandardPool, Task};

/// Task that sleeps for a specified duration
struct SleepTask {
    id: u32,
    sleep_ms: u64,
}

impl Task for SleepTask {
    fn execute(&self) {
        println!("Task {} starting", self.id);
        thread::sleep(Duration::from_millis(self.sleep_ms));
        println!("Task {} completed", self.id);
    }
}

fn main() {
    println!("Creating thread pool with 3 workers");
    let pool = StandardPool::new(3);

    // Create 5 tasks with different sleep durations
    let tasks = vec![
        Arc::new(SleepTask {
            id: 1,
            sleep_ms: 1000,
        }),
        Arc::new(SleepTask {
            id: 2,
            sleep_ms: 500,
        }),
        Arc::new(SleepTask {
            id: 3,
            sleep_ms: 1500,
        }),
        Arc::new(SleepTask {
            id: 4,
            sleep_ms: 800,
        }),
        Arc::new(SleepTask {
            id: 5,
            sleep_ms: 200,
        }),
    ];

    println!("Submitting tasks...");
    for task in tasks {
        pool.add_task(task);
    }

    println!("Waiting for all tasks to complete...");
    pool.barrier();
    println!("All tasks completed!");
}
