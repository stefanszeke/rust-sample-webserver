use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 1..=size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {

        // as_ref to borrow and not take ownership of the sender
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();   // we send the job (closure/fn) to the worker
                                                            // the worker holding the receiver will receive the job,
                                                            // let go of the lock and execute the job
                                                            // while the sender can send another job to the receiver
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>, // make it optional so others can take ownership of the thread

                                            // the take method on Option takes the Some variant out and leaves None in its place.
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv(); // lock so only one worker can get a job at a time,

            // lock is already released here

            match message {
                Ok(job) => {
                    println!("⚙️ Worker {id} got a job; executing.");
                    job();
                    // next iteration after the job is done
                }
                Err(_) => {
                    println!("⚙️ Worker {id} got a termination signal; shutting down.");
                    break;
                }
            }

        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            // if the worker has a thread, take it and join it (wait for it to finish, return the result)
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
