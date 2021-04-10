use std::{
    time::Instant,
    thread,
};

#[macro_use]
extern crate lazy_static; 

mod coro;

use coro::{
    Executor,
    //Reactor,
    //AsyncCall,
    syscall_wait,
    close_runtime,
};

fn main() {
    //let future1 = ||
    /*
    let reactor = Reactor::new();
    let reactor1 = reactor.clone();
    let reactor2 = reactor.clone();
    */
    let start = Instant::now();
    let start2 = Instant::now();
    //let future1 = move || wait_secs(reactor1, 1, 1, start);
    //let future2 = move || wait_secs(reactor2, 2, 2, start2);
    let future1 = move || wait_secs(2, start);
    let future2 = move || wait_secs(1, start2);
    let th = thread::spawn(move || {
        
        //
        
        /*
        let fut1 = async {
            let val = AsyncCall::new(reactor_clone1, 1, 1).await;
            println!("Got {} at time: {:.2}.", val, start1.elapsed().as_secs_f32());
        };
        let reactor_clone2 = reactor.clone();
        let fut2 = async {
            let val = AsyncCall::new(reactor_clone2, 2, 2).await;
            println!("Got {} at time: {:.2}.", val, start2.elapsed().as_secs_f32());
        };
        */
        let mut executor = Executor::new();
        //executor.spawn(wait_secs(reactor.clone(), 1, 1, start));
        //executor.spawn(wait_secs(reactor.clone(), 2, 2, start));
        executor.push(future1());
        executor.push(future2());
        executor.run();
        //thread::spawn(move || executor.run());
        //block_on(mainfut);
        
    });
    println!("do other things here");
    th.join().unwrap();
    close_runtime();
}

async fn wait_secs(secs:u64, start_time:Instant) {
    let val = syscall_wait(secs).await;
    println!("Got {} at time: {:.2}.", val, start_time.elapsed().as_secs_f32());
}
/*
use std::{
    future::Future, sync::{ mpsc::{channel, Sender}, Arc, Mutex, Condvar},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}, mem, pin::Pin,
    thread::{self, JoinHandle}, time::{Duration, Instant}, collections::HashMap
};
*/
/*
async fn wait_secs(reactor:Arc<Mutex<Box<Reactor>>>, secs:u64, id:usize, start_time:Instant) {
    let val = AsyncCall::new(reactor, secs, id).await;
    println!("Got {} at time: {:.2}.", val, start_time.elapsed().as_secs_f32());
}
*/
