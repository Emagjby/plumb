use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE_OUTPUT: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(enabled: bool) {
    VERBOSE_OUTPUT.store(enabled, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE_OUTPUT.load(Ordering::Relaxed)
}
