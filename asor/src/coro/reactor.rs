use std::{
    thread::{
        self,
        JoinHandle,
    },
    sync::{
        Arc,
        Mutex,
        mpsc::{channel, Sender},
        atomic::{AtomicU64, Ordering},
    },
    task::{
        Waker,
        
    },
    time::Duration,
    collections::HashMap,
    mem,
};

pub struct Reactor {
    dispatcher: Sender<ReactorEvent>,
    tasks: HashMap<usize, CallState>,
    //目前的syscall实际是“新建线程sleep”,所以需要JoinHandle(),放在rCore上后应删去
    handle: Option<JoinHandle<()>>,
}

#[derive(Debug)]
enum ReactorEvent {
    Close,
    Timeout(u64, usize),
}

pub enum CallState {
    Ready,
    Finished,
    Waiting(Waker),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallId(u64);

impl CallId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        CallId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    pub fn to(&self) -> usize {
        self.0 as usize
    }
}

impl Reactor {
    pub fn new() -> Arc<Mutex<Box<Self>>> {
        let (tx, rx) = channel::<ReactorEvent>();
        let reactor = Arc::new(Mutex::new(Box::new(Reactor {
            dispatcher: tx,
            handle: None,
            tasks: HashMap::new(),
        })));
        
        let reactor_clone = Arc::downgrade(&reactor);
        let handle = thread::spawn(move || {
            let mut handles = vec![];
            for event in rx {
                let reactor = reactor_clone.clone();
                match event {
                    ReactorEvent::Close => break,
                    ReactorEvent::Timeout(duration, id) => {
                        let event_handle = thread::spawn(move || {
                            thread::sleep(Duration::from_secs(duration));
                            let reactor = reactor.upgrade().unwrap();
                            reactor.lock().map(|mut r| r.wake(id)).unwrap();
                        });
                        handles.push(event_handle);
                    }
                }
            }
            handles.into_iter().for_each(|handle| handle.join().unwrap());
        });
        reactor.lock().map(|mut r| r.handle = Some(handle)).unwrap();
        reactor
    }

    pub fn wake(&mut self, id: usize) {
        let state = self.tasks.get_mut(&id).unwrap();
        match mem::replace(state, CallState::Ready) {
            CallState::Waiting(waker) => waker.wake(),
            CallState::Finished => panic!("Called 'wake' twice on task: {}", id),
            _ => unreachable!()
        }
    }
    pub fn finish(&mut self, id: usize) {
        *self.tasks.get_mut(&id).unwrap() = CallState::Finished;
    }

    pub fn has_task_or_insert(&mut self, id:usize, waker:Waker) -> bool {
        if self.tasks.contains_key(&id) {
            self.tasks.insert(id, CallState::Waiting(waker));
            true
        } else {
            false
        }
    }

    pub fn register(&mut self, duration: u64, waker: Waker, id: usize) {
        if self.tasks.insert(id, CallState::Waiting(waker)).is_some() {
            panic!("Tried to insert a task with id: '{}', twice!", id);
        }
        self.dispatcher.send(ReactorEvent::Timeout(duration, id)).unwrap();
    }

    pub fn close(&mut self) {
        self.dispatcher.send(ReactorEvent::Close).unwrap();
    }

    pub fn is_ready(&self, id: usize) -> bool {
        self.tasks.get(&id).map(|state| match state {
            CallState::Ready => true,
            _ => false,
        }).unwrap_or(false)
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        self.handle.take().map(|h| h.join().unwrap()).unwrap();
    }
}
