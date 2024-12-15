# Rust Thread Pool

A simple thread pool implementation in Rust.

## Features

- Fixed-size thread pool
- Custom task support via `Task` trait
- Thread-safe task queue
- Barrier synchronization
- Graceful shutdown

## Usage Example

Here's a simple example showing how to create tasks that sleep for different durations:

```rust
use std::sync::Arc;
use std::{thread, time::Duration};
use threadpools::standardpool::{StandardPool, Task};

// Define a task that sleeps
struct SleepTask {
    name: String,
    sleep_ms: u64,
}

impl Task for SleepTask {
    fn execute(&self) {
        println!("🛏️ {} is going to sleep for {}ms", self.name, self.sleep_ms);
        thread::sleep(Duration::from_millis(self.sleep_ms));
        println!("⭐ {} woke up!", self.name);
    }
}

fn main() {
    // Create a thread pool with 2 workers
    let pool = StandardPool::new(2);

    // Create several tasks with different sleep times
    let tasks = vec![
        Arc::new(SleepTask {
            name: "Alice".to_string(),
            sleep_ms: 1000,
        }),
        Arc::new(SleepTask {
            name: "Bob".to_string(),
            sleep_ms: 500,
        }),
        Arc::new(SleepTask {
            name: "Charlie".to_string(),
            sleep_ms: 1500,
        }),
    ];

    // Submit tasks to the pool
    for task in tasks {
        pool.add_task(task);
    }

    // Wait for all tasks to complete
    pool.barrier();
    println!("✅ All tasks completed!");
}
```

Sample output:
```
🛏️ Alice is going to sleep for 1000ms
🛏️ Bob is going to sleep for 500ms
⭐ Bob woke up!
🛏️ Charlie is going to sleep for 1500ms
⭐ Alice woke up!
⭐ Charlie woke up!
✅ All tasks completed!
```

## Running Examples

```bash
# Simple sleep tasks
cargo run --example simple_sleep

# Parallel sum computation
cargo run --example parallel_sum
```

## Requirements

- Rust 1.56.0 or later

## License

MIT License - See [LICENSE](LICENSE) for details.
