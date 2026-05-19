//! Integration tests for FileWatcher (requires `--features watching`).
//!
//! Debounce window is 1 second; tests wait up to 3 seconds total.

#[cfg(feature = "watching")]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use todotxt_core::FileWatcher;

    /// Wait up to `max_ms` milliseconds for `flag` to become `true`, polling every 100ms.
    fn wait_for_flag(flag: &Arc<AtomicBool>, max_ms: u64) -> bool {
        let polls = max_ms / 100;
        for _ in 0..polls {
            if flag.load(Ordering::SeqCst) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        flag.load(Ordering::SeqCst)
    }

    #[test]
    fn watcher_fires_callback_on_file_write() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let fired = Arc::new(AtomicBool::new(false));
        let fired_clone = Arc::clone(&fired);

        let _watcher = FileWatcher::new(
            &path,
            Arc::new(move || {
                fired_clone.store(true, Ordering::SeqCst);
            }),
        )
        .expect("FileWatcher::new should succeed for a valid path");

        std::fs::write(&path, "Buy milk\nSend email\n").unwrap();

        let did_fire = wait_for_flag(&fired, 3000);
        assert!(
            did_fire,
            "FileWatcher callback did not fire within 3 seconds after file write"
        );
    }

    #[test]
    fn watcher_stop_does_not_panic() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let watcher =
            FileWatcher::new(tmp.path(), Arc::new(|| {})).expect("FileWatcher::new should succeed");
        watcher.stop();
    }

    #[test]
    fn watcher_fires_on_atomic_write_rename() {
        // Simulates TaskList::save() pattern: write to temp file, then rename over original
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("todo.txt");
        std::fs::write(&target, "Initial content\n").unwrap();

        let fired = Arc::new(AtomicBool::new(false));
        let fired_clone = Arc::clone(&fired);

        let _watcher = FileWatcher::new(
            &target,
            Arc::new(move || {
                fired_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

        let tmp_path = dir.path().join(".todo.txt.tmp");
        std::fs::write(&tmp_path, "Updated content\n").unwrap();
        std::fs::rename(&tmp_path, &target).unwrap();

        let did_fire = wait_for_flag(&fired, 3000);
        assert!(
            did_fire,
            "FileWatcher callback did not fire after atomic rename within 3 seconds"
        );
    }
}
