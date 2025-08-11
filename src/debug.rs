// Global flag to control DEBUG output
use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(true);

// Function to safely check if DEBUG output is enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

// Function to set DEBUG output enabled/disabled
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

// Macro to conditionally print DEBUG messages
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::debug::is_debug_enabled() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug_eprintln {
    ($($arg:tt)*) => {
        if $crate::debug::is_debug_enabled() {
            eprintln!($($arg)*);
        }
    };
}
