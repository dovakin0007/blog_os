use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Poll, Context};

pub mod simple_executor;

pub struct Task{
    future: Pin<Box<dyn Future <Output = ()>>>
}

impl Task{
    // the function Takes in which implements future amd pins it to memeory and returns task and it needs a static lifetime 
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task { future: Box::pin(future) }
    }

    // takes in Pin<mut T> so we convert future.as_mut represent Pin::as_mut covert into Pin<Box<T>> this method is called by executor so it private
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }


    
}