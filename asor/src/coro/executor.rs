use std::{collections::BTreeMap, task::Wake, boxed::Box, pin::Pin};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::future::Future;

use crossbeam_queue::ArrayQueue;

use super::{
    task::{Task, TaskId},
    waker::TaskWaker,
    tools::Queue,
};

const MAX_QUEUE_SIZE:usize = 100;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    todo_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            todo_queue: Arc::new(ArrayQueue::new(MAX_QUEUE_SIZE)),
            waker_cache: BTreeMap::new(),
        }
    }
    
    fn spawn_task(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.todo_queue.push(task_id).expect("queue full");
    }
    
    pub fn push(&mut self, future: impl Future<Output = ()> + 'static) {
        self.spawn_task(Task::new(future));
    }
    pub fn run(&mut self) -> bool {
        while self.tasks.is_empty() == false {
            self.run_ready_tasks();
            //self.sleep_if_idle();
        }
        true
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        /*
        let Self {
            tasks,
            todo_queue,
            waker_cache,
        } = self;
*/
        let queue_clone = self.todo_queue.clone();
        while let Ok(task_id) = self.todo_queue.pop() {
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                //None => continue,
                None => panic!("waker.wake() is called twice or more!"),
            };
            let waker = self.waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, queue_clone.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
/*
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.todo_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
    */
}
