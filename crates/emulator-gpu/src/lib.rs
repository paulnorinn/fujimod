//! Shared GPU context, coverage rasterization, resident surfaces, and guest-memory transfers.

#![recursion_limit = "256"]

#[cfg(not(target_os = "emscripten"))]
mod coverage;
#[cfg(not(target_os = "emscripten"))]
mod surfaces;
#[cfg(not(target_os = "emscripten"))]
pub use coverage::{
    CoverageBatch, CoverageEdge, CoverageKernel, CoverageRaster, CoverageRequest, EdgeCoordinates,
    FillRule, SampleGrid,
};
#[cfg(not(target_os = "emscripten"))]
pub use surfaces::{
    GpuContext, GpuSurface, PendingSurfaceReadbacks, PixelWriteMask, SurfaceExtent, SurfaceFormat,
    SurfaceKey, SurfaceLayout, SurfaceReadback, context, context_async,
};

/// The tests share one process-wide GPU context, so they take turns: concurrent
/// uploads and read-backs on one device interleave and observe each other's
/// bytes. A real session is single-threaded and never needs this.
#[cfg(all(test, not(target_os = "emscripten")))]
pub(crate) mod test_lock {
    use std::sync::{Mutex, MutexGuard};

    static LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn guard() -> MutexGuard<'static, ()> {
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }
}
