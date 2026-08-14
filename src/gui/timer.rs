use std::collections::HashMap;
use std::sync::Weak;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use egui::mutex::Mutex;

pub struct TimerManager {
    ctx: egui::Context,
    workers: HashMap<Duration, Worker>,
}

impl TimerManager {
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            ctx,
            workers: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
        for worker in self.workers.values_mut() {
            for handle in worker.handles.lock().iter_mut() {
                if let Some(handle) = handle.upgrade() {
                    let state = handle.pending_state.load(Ordering::Relaxed);
                    handle.active_state.store(state, Ordering::Relaxed);
                    handle.pending_state.store(false, Ordering::Relaxed);
                }
            }
        }
    }

    /// Start a new timer of the specific `duration`
    ///
    /// Timer will be stopped after the return value dropped (i.e. reference count become zero)
    pub fn start(&mut self, duration: Duration) -> Arc<TimerHandle> {
        let worker = self
            .workers
            .entry(duration)
            .or_insert_with(|| Worker::new(self.ctx.clone(), duration));
        worker.new_handle()
    }
}

#[derive(Default)]
pub struct TimerHandle {
    pending_state: AtomicBool,
    active_state: AtomicBool,
}

impl TimerHandle {
    #[inline]
    pub fn timeout(&self) -> bool {
        self.active_state.load(Ordering::Relaxed)
    }
}

struct Worker {
    duration: Duration,

    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,

    handles: Arc<Mutex<Vec<Weak<TimerHandle>>>>,
}

impl Worker {
    fn new(ctx: egui::Context, duration: Duration) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();

        let handles = Arc::new(Mutex::new(Vec::<Weak<TimerHandle>>::new()));
        let handles_clone = handles.clone();

        let thread = thread::spawn(move || {
            loop {
                thread::sleep(duration);
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                let mut modified = false;
                for handle in handles_clone.lock().iter() {
                    if let Some(handle) = handle.upgrade() {
                        handle.pending_state.store(true, Ordering::Relaxed);
                        modified = true;
                    }
                }
                if modified {
                    ctx.request_repaint();
                }
            }
        });

        Self {
            duration,
            stop,
            thread: Some(thread),
            handles,
        }
    }

    fn new_handle(&mut self) -> Arc<TimerHandle> {
        let mut handles = self.handles.lock();

        // Clean the expired handles
        handles.retain(|x| x.strong_count() > 0);

        // Create the new handle
        let handle = Arc::new(TimerHandle::default());
        handles.push(Arc::downgrade(&handle));

        handle
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
