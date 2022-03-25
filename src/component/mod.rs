use std::sync::atomic::{AtomicBool, Ordering};

pub struct Control {
    pub is_paused: AtomicBool,
}

impl Control {
    pub fn new() -> Control {
        Control { is_paused: AtomicBool::new(false) }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    pub fn start(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::Relaxed);
    }
}

pub trait Controllable {
    fn control(&self) -> &Control;
}