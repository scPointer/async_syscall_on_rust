use std::{
    sync::{
        Arc,
        Mutex,
    },
};

use super::{
    Reactor,
    AsyncCall,
    CallId,
};

lazy_static!{
    pub static ref REACTOR:Arc<Mutex<Box<Reactor>>> = Reactor::new();
}

pub fn syscall_wait(secs: u64) -> AsyncCall {
    let r = REACTOR.clone();
    AsyncCall::new(r, secs, CallId::new().to())
}

pub fn close_runtime() {
    REACTOR.lock().map(|mut r| r.close()).unwrap();
}