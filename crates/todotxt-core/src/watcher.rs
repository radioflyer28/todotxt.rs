//! File watching with debounce.
//!
//! Enabled with Cargo feature `watching`. The watcher monitors a single file
//! by watching its parent directory (more reliable for atomic writes via rename).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

use crate::error::TodoError;

/// A file watcher that fires a callback after a 1-second debounce window.
///
/// Watching is implemented by monitoring the **parent directory** with
/// `RecursiveMode::NonRecursive` and filtering events to the target filename.
/// This pattern is reliable even when the file is replaced atomically via rename
/// (as `TaskList::save()` does).
///
/// The background thread stops when `FileWatcher` is dropped or when `stop()` is called.
pub struct FileWatcher {
    // Holds the debouncer alive. The background thread stops when this is dropped.
    _debouncer: Debouncer<RecommendedWatcher>,
}

impl FileWatcher {
    /// Create a new file watcher for `path`.
    ///
    /// `callback` is called on the background debouncer thread after each
    /// 1-second quiet period following a file change. The callback receives no
    /// arguments — the caller should call `task_list.reload()` to re-read the file.
    ///
    /// Returns `TodoError::Watch` if the underlying `notify` watcher fails to start.
    pub fn new(
        path: impl AsRef<Path>,
        callback: Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Self, TodoError> {
        let path = path.as_ref();

        // Watch the parent directory, not the file directly.
        // File-level watching is unreliable for atomic writes (inode changes on rename).
        let parent = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        // Capture the target filename to filter events from sibling files.
        let target_name = path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();

        let cb = Arc::clone(&callback);

        let mut debouncer = new_debouncer(
            Duration::from_secs(1),
            move |res: DebounceEventResult| {
                if let Ok(events) = res {
                    // Filter to events for our specific file only.
                    if events
                        .iter()
                        .any(|e| e.path.file_name() == Some(target_name.as_os_str()))
                    {
                        cb();
                    }
                }
                // Ignore watcher errors — they are transient.
            },
        )?;

        debouncer
            .watcher()
            .watch(&parent, RecursiveMode::NonRecursive)?;

        Ok(FileWatcher {
            _debouncer: debouncer,
        })
    }

    /// Stop the watcher explicitly.
    ///
    /// The background thread is also stopped when `FileWatcher` is dropped, so
    /// calling `stop()` is optional but useful for explicit lifecycle control.
    pub fn stop(self) {
        drop(self);
    }
}
