use std::{
    sync::{
        Arc,
        Mutex,
    },
    task::{
        Poll,
        Context,
    },
    future::Future,
    boxed::Box,
    pin::Pin,
};

use super::{
    reactor::{
        Reactor,
        CallState,
        CallId,
    },
};

#[derive(Clone)]
pub struct AsyncCall {
    id: usize,
    reactor: Arc<Mutex<Box<Reactor>>>,
    data: u64,
}

impl AsyncCall {
    pub fn new(reactor: Arc<Mutex<Box<Reactor>>>, data: u64, id: usize) -> Self {
        AsyncCall { id, reactor, data }
    }
}

impl Future for AsyncCall {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut r = self.reactor.lock().unwrap();
        if r.is_ready(self.id) {
            //*r.tasks.get_mut(&self.id).unwrap() = CallState::Finished;
            r.finish(self.id);
            Poll::Ready(self.id)
        //} else if r.tasks.contains_key(&self.id) {
        } else if r.has_task_or_insert(self.id, cx.waker().clone()) {
            //r.tasks.insert(self.id, CallState::NotReady(cx.waker().clone()));
            Poll::Pending
        } else {
            r.register(self.data, cx.waker().clone(), self.id);
            Poll::Pending
        }
    }
}
