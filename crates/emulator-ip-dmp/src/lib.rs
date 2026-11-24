//! DMP intellectual-property implementations.

#![recursion_limit = "256"]

pub mod fill;
pub mod gles;
pub mod pica;
pub mod smaph;

#[cfg(target_os = "emscripten")]
static WEBGPU_BRIDGE_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Select asynchronous host WebGPU rendering for DMP command lists.
pub fn configure_webgpu_bridge(enabled: bool) {
    #[cfg(target_os = "emscripten")]
    WEBGPU_BRIDGE_ENABLED.store(enabled, std::sync::atomic::Ordering::SeqCst);
    #[cfg(not(target_os = "emscripten"))]
    let _ = enabled;
}

pub(crate) fn webgpu_bridge_enabled() -> bool {
    #[cfg(target_os = "emscripten")]
    {
        WEBGPU_BRIDGE_ENABLED.load(std::sync::atomic::Ordering::SeqCst)
    }
    #[cfg(not(target_os = "emscripten"))]
    {
        false
    }
}

/// The GPU tests share one process-wide device context, so they take turns:
/// concurrent uploads and read-backs on one device would observe each other's
/// bytes. A real command list is rendered single-threaded and needs no lock.
#[cfg(test)]
pub(crate) mod test_lock {
    use std::sync::{Mutex, MutexGuard};

    static LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn guard() -> MutexGuard<'static, ()> {
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }
}
