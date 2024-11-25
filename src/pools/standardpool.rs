// SPDX-License-Identifier: MIT

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// Trait defining a general task structure, with an execute method to be
/// implemented by each task.
pub trait Task {
    fn execute(&self);
}

/// Task implementation that represents a barrier synchronization point.
/// Each worker thread must reach the barrier before any are allowed to proceed.
struct BarrierTask {
    /// Count of arrivals and barrier condition.
    sync_point: Arc<(Mutex<usize>, Condvar)>,
    /// Number of workers required to reach the barrier.
    total_workers: usize,
}

impl Task for BarrierTask {
    fn execute(&self) {
        let (ref arrival_count, ref barrier_condvar) = *self.sync_point;
        let mut count_guard = arrival_count.lock().unwrap();

        // Increment arrival count to signal that this worker has reached the
        // barrier.
        *count_guard += 1;

        // Notify all if this is the last worker to arrive.
        if *count_guard == self.total_workers {
            barrier_condvar.notify_all();
            return;
        }

        // Otherwise, wait for all workers to reach the barrier.
        while *count_guard != self.total_workers {
            count_guard = barrier_condvar.wait(count_guard).unwrap();
        }
    }
}

/// Shared data structure for the job scheduler, holding the task queue and a
/// shutdown flag.
struct SchedulerSharedData {
    /// Queue of tasks for worker threads.
    task_queue: VecDeque<Arc<dyn Task + Send + Sync>>,
    /// Flag to signal shutdown to worker threads.
    shutdown_flag: bool,
}

/// Job scheduler that manages worker threads, tasks, and synchronization for
/// task execution.
pub struct JobScheduler {
    /// Pool of worker threads.
    worker_threads: Vec<thread::JoinHandle<()>>,
    /// Shared data protected by a mutex
    shared_data: Arc<Mutex<SchedulerSharedData>>,
    /// Condition variable to notify workers of new tasks or shutdown.
    task_available: Arc<Condvar>,
}

impl SchedulerSharedData {
    fn new() -> Self {
        SchedulerSharedData {
            task_queue: VecDeque::with_capacity(128),
            shutdown_flag: false,
        }
    }
}

impl JobScheduler {
    /// Creates a new job scheduler with the specified number of worker threads.
    pub fn new(thread_count: usize) -> Self {
        let mut job_scheduler = JobScheduler {
            worker_threads: Vec::with_capacity(thread_count),
            shared_data: Arc::new(Mutex::new(SchedulerSharedData::new())),
            task_available: Arc::new(Condvar::new()),
        };

        // Spawn worker threads that wait for tasks to be added to the task
        // queue.
        for _ in 0..thread_count {
            let shared_data = job_scheduler.shared_data.clone();
            let task_condvar = job_scheduler.task_available.clone();
            job_scheduler
                .worker_threads
                .push(thread::spawn(move || loop {
                    let task = {
                        // Acquire the lock to access shared data
                        let mut shared_data_guard = match shared_data.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                shared_data.clear_poison();
                                poisoned.into_inner()
                            }
                        };

                        // Wait until there is a task to process or a shutdown
                        // signal.
                        while shared_data_guard.task_queue.is_empty()
                            && !shared_data_guard.shutdown_flag
                        {
                            shared_data_guard = task_condvar
                                .wait(shared_data_guard)
                                .unwrap_or_else(|e| e.into_inner());
                        }

                        // Break the loop if shutdown flag is set.
                        if shared_data_guard.shutdown_flag {
                            break;
                        }

                        // Remove the task from the front of the queue.
                        shared_data_guard.task_queue.pop_front().unwrap()
                    };

                    // Execute the task outside the lock scope.
                    task.execute();
                }));
        }

        job_scheduler
    }

    /// Adds a task to the scheduler's task queue and notifies one waiting
    /// worker thread.
    pub fn add_task(&self, task: Arc<dyn Task + Send + Sync>) {
        let mut shared_data_guard = match self.shared_data.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                self.shared_data.clear_poison();
                poisoned.into_inner()
            }
        };
        shared_data_guard.task_queue.push_back(task);
        self.task_available.notify_one();
    }

    /// Creates a barrier task that synchronizes all workers before continuing.
    pub fn barrier(&self) {
        let shared_sync_point = Arc::new((Mutex::new(0), Condvar::new()));
        let total_workers = self.worker_threads.len() + 1;

        // Add barrier tasks for each worker thread.
        for _ in 0..total_workers - 1 {
            self.add_task(Arc::new(BarrierTask {
                sync_point: shared_sync_point.clone(),
                total_workers,
            }));
        }

        // Main thread reaches the barrier by executing its own BarrierTask.
        let main_barrier_task = BarrierTask {
            sync_point: shared_sync_point.clone(),
            total_workers,
        };
        main_barrier_task.execute();
    }
}

impl Drop for JobScheduler {
    /// Signals the scheduler to shut down by setting the shutdown flag and
    /// notifying all workers.
    fn drop(&mut self) {
        {
            let mut shared_data_guard = match self.shared_data.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    self.shared_data.clear_poison();
                    poisoned.into_inner()
                }
            };
            shared_data_guard.shutdown_flag = true;
            self.task_available.notify_all();
        }

        for handler in self.worker_threads.drain(..) {
            let _ = handler.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::tasks::{get_subslice_sum_tasks, SharedCounterTask};
    use std::sync::atomic::Ordering::SeqCst;

    #[test]
    fn test_functionality_shared_variable() {
        // Set up test parameters
        let num_workers = 32;
        let total_tasks = 2048;
        let scheduler = JobScheduler::new(num_workers);
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
        let scheduler = JobScheduler::new(num_workers);

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
}
