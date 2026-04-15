use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("task not found at index {index}")]
    NotFound { index: usize },

    #[error("index {index} out of bounds (task count: {count})")]
    IndexOutOfBounds { index: usize, count: usize },

    #[cfg(feature = "watching")]
    #[error("file watcher error: {0}")]
    Watch(#[from] notify_debouncer_mini::notify::Error),
}
