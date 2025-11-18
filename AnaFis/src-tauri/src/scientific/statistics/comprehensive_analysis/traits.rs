//! Traits for analysis operations.

/// Progress callback trait for long-running operations
pub trait ProgressCallback: Send + Sync + 'static {
    fn report_progress(&self, current: usize, total: usize, message: &str);
}

/// Optional progress callback that does nothing
pub struct NoOpProgressCallback;

impl ProgressCallback for NoOpProgressCallback {
    fn report_progress(&self, _current: usize, _total: usize, _message: &str) {
        // Do nothing
    }
}