use super::Task;
use alloc::collections::VecDeque;
use core::task::{Waker, RawWaker};
use core::task::RawWakerVTable;
use core::task::{Context, Poll};
// this struct contains task_queue which has a VecDeque type
pub struct SimpleExecutor {
    task_queue : VecDeque<Task>
}

impl SimpleExecutor {
    // creates a new VecDeque
    pub fn new() -> Self{
        SimpleExecutor { task_queue: VecDeque::new() }
    }

    //spawn method adds in a new task into vec queue it works by FIFO order
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self){
        //iterating through all tasks in queue by matchig while let
        while let Some(mut task) = self.task_queue.pop_front(){
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);//creating a context type by wrapping in the waker method
            match task.poll(&mut context){ //invoking the poll if the method returns Poll::Ready the task is finished we continuie
                Poll::Ready(()) => {}//task is done
                Poll::Pending => self.task_queue.push_back(task),// else the task is added back into queue
            } 
        }
    }
}

fn dummy_raw_waker()-> RawWaker {
    fn no_op(_: *const()){}
    fn clone(_: *const()) -> RawWaker {
        dummy_raw_waker()
    }
    //RawWakerTable contains the funtions that need to executed on the wake clone or drop
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const(), vtable)
}

fn dummy_waker() -> Waker{
    unsafe {
        Waker::from_raw(dummy_raw_waker())
    }
}