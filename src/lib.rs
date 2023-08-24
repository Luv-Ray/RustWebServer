use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    threads: Vec<Thread>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut threads = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            threads.push(Thread::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            threads,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .expect("Can't send job!");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for thread in self.threads.drain(..) {
            println!("Shutting down thread {}", thread.id);

            thread.handle.join().unwrap();
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Thread {
    id: usize,
    handle: thread::JoinHandle<()>,
}

impl Thread {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Thread {
        let handle = thread::spawn(move || loop {
            if let Ok(job) = receiver.lock().unwrap().recv() {
                println!("Thread {id} got a job; executing.");
                job();
            } else {
                println!("Thread {id} disconnected; shutting down.");
                break;
            }
        });

        Thread { id, handle }
    }
}
