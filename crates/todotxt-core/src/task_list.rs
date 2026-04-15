use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;

use crate::error::TodoError;
use crate::filter::Filter;
use crate::sort::SortOrder;
use crate::task::Task;

// ── LineEnding ────────────────────────────────────────────────────────────────

/// The line-ending style detected in a todo.txt file.
///
/// Used to preserve the original file's line endings when saving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    /// Returns the line-ending as a string slice.
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        }
    }
}

// ── TaskList ──────────────────────────────────────────────────────────────────

/// A list of tasks backed by a todo.txt file.
///
/// All mutations (add, update, delete) immediately persist to disk using an
/// atomic write (`tempfile::NamedTempFile::persist()`) so the file is never
/// left in a partially-written state.
pub struct TaskList {
    path: PathBuf,
    tasks: Vec<Task>,
    line_ending: LineEnding,
    preserve_whitespace: bool,
}

impl TaskList {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Load a `TaskList` from a todo.txt file.
    ///
    /// - Strips UTF-8 BOM (`\u{FEFF}`) from the first line (fixes C-3).
    /// - Detects CRLF vs LF by scanning the first 4000 bytes (matches C# algorithm).
    /// - Skips blank lines by default (`preserve_whitespace: false`).
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TodoError> {
        Self::load_with_options(path, false)
    }

    /// Load a `TaskList` with configurable whitespace preservation.
    pub fn load_with_options(
        path: impl AsRef<Path>,
        preserve_whitespace: bool,
    ) -> Result<Self, TodoError> {
        let path = path.as_ref().to_path_buf();

        let content = fs::read_to_string(&path).map_err(|source| TodoError::Io {
            path: path.clone(),
            source,
        })?;

        // Strip UTF-8 BOM if present (fixes C-3: BOM breaks parsing).
        let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

        // Detect line ending by scanning the first 4000 bytes (matches C# algorithm).
        let line_ending = detect_line_ending(content);

        // Split and parse lines.
        let tasks = split_lines(content, line_ending)
            .filter(|line| preserve_whitespace || !line.trim().is_empty())
            .map(Task::parse)
            .collect();

        Ok(TaskList {
            path,
            tasks,
            line_ending,
            preserve_whitespace,
        })
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    /// Save all tasks to disk atomically via `tempfile::NamedTempFile::persist()`.
    ///
    /// The file is NEVER truncated mid-write (fixes C-2). If the process is killed
    /// during a save, either the old or new file is present — never a partial write.
    ///
    /// Line endings are preserved (CRLF or LF) from the original file (fixes C-4).
    /// UTF-8 BOM is NOT written even if the original file had one.
    pub fn save(&self) -> Result<(), TodoError> {
        let parent = self.path.parent().unwrap_or(Path::new("."));

        // Create a temp file in the SAME directory as the target so
        // persist() is guaranteed to be an atomic rename (not a copy).
        let mut temp = NamedTempFile::new_in(parent).map_err(|source| TodoError::Io {
            path: parent.to_path_buf(),
            source,
        })?;

        // Write all tasks joined by the detected line ending.
        let line_sep = self.line_ending.as_str();
        let content: String = self
            .tasks
            .iter()
            .map(|t| t.to_raw())
            .collect::<Vec<_>>()
            .join(line_sep);

        // Always end with a trailing line separator.
        let content = if content.is_empty() {
            String::new()
        } else {
            format!("{}{}", content, line_sep)
        };

        temp.write_all(content.as_bytes())
            .map_err(|source| TodoError::Io {
                path: parent.to_path_buf(),
                source,
            })?;

        temp.flush().map_err(|source| TodoError::Io {
            path: parent.to_path_buf(),
            source,
        })?;

        // Ensure all data is flushed to the OS before the atomic rename.
        temp.as_file()
            .sync_all()
            .map_err(|source| TodoError::Io {
                path: parent.to_path_buf(),
                source,
            })?;

        // Atomic rename — replaces the original file in a single syscall.
        temp.persist(&self.path)
            .map_err(|e| TodoError::Io {
                path: self.path.clone(),
                source: e.error,
            })?;

        Ok(())
    }

    /// Re-read the file from disk, updating all in-memory state.
    pub fn reload(&mut self) -> Result<(), TodoError> {
        let updated = Self::load_with_options(&self.path, self.preserve_whitespace)?;
        self.tasks = updated.tasks;
        self.line_ending = updated.line_ending;
        Ok(())
    }

    // ── CRUD ──────────────────────────────────────────────────────────────────

    /// Append a task and save.
    pub fn add(&mut self, task: Task) -> Result<(), TodoError> {
        self.tasks.push(task);
        self.save()
    }

    /// Replace the task at `index` and save.
    ///
    /// Returns `TodoError::IndexOutOfBounds` if `index >= tasks.len()`.
    pub fn update(&mut self, index: usize, new_task: Task) -> Result<(), TodoError> {
        let count = self.tasks.len();
        if index >= count {
            return Err(TodoError::IndexOutOfBounds { index, count });
        }
        self.tasks[index] = new_task;
        self.save()
    }

    /// Remove the task at `index` and save.
    ///
    /// Uses **index-based identity** — not raw-string comparison (fixes C-1).
    /// Given two identical tasks at indices 0 and 1, `delete(0)` removes only
    /// the first one.
    ///
    /// Returns `TodoError::IndexOutOfBounds` if `index >= tasks.len()`.
    pub fn delete(&mut self, index: usize) -> Result<(), TodoError> {
        let count = self.tasks.len();
        if index >= count {
            return Err(TodoError::IndexOutOfBounds { index, count });
        }
        self.tasks.remove(index);
        self.save()
    }

    // ── Getters ───────────────────────────────────────────────────────────────

    /// Returns the tasks as a slice (not `&Vec<Task>`).
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Returns the number of tasks.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Returns `true` if there are no tasks.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns the path to the backing file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the detected line ending style.
    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    // ── Filtering, Sorting, Batch ─────────────────────────────────────────────

    /// Return all tasks that match `filter`, paired with their indices.
    ///
    /// Does NOT mutate or save. The index can be used for subsequent `update()` or
    /// `delete()` calls without requiring a second lookup.
    pub fn filter(&self, filter: &Filter) -> Vec<(usize, &Task)> {
        self.tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| filter.matches(task))
            .collect()
    }

    /// Sort tasks in-place according to `order` using a stable sort.
    ///
    /// Tasks that compare equal preserve their original relative order
    /// (matching LINQ `OrderBy` behavior from the C# reference implementation).
    ///
    /// Does NOT save to disk — call `save()` explicitly if persistence is needed.
    pub fn sort(&mut self, order: SortOrder) {
        self.tasks.sort_by(|a, b| order.compare(a, b));
    }

    /// Replace multiple tasks atomically — validates all indices first, then
    /// applies all replacements, then calls `save()` exactly once.
    ///
    /// **Fail-fast:** if ANY index is out of bounds, returns `IndexOutOfBounds`
    /// immediately and NO tasks are mutated.
    pub fn batch_update(&mut self, replacements: Vec<(usize, Task)>) -> Result<(), TodoError> {
        let count = self.tasks.len();
        // Validate ALL indices before ANY mutation
        for &(index, _) in &replacements {
            if index >= count {
                return Err(TodoError::IndexOutOfBounds { index, count });
            }
        }
        // Apply all replacements in a single pass
        for (index, new_task) in replacements {
            self.tasks[index] = new_task;
        }
        // Single save() call — avoids N disk writes for N tasks
        self.save()
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Detect the line ending used in the content by scanning the first 4000 bytes.
///
/// Matches the C# `GetPreferredFileLineEndingFromFile()` algorithm:
/// - If `\r\n` is found → `CrLf`
/// - If only `\n` is found → `Lf`
/// - If no newline found → `Lf` (Unix default)
fn detect_line_ending(content: &str) -> LineEnding {
    let bytes = content.as_bytes();
    let scan_len = bytes.len().min(4000);
    let mut prev = b'\0';

    for &b in &bytes[..scan_len] {
        if b == b'\n' {
            return if prev == b'\r' {
                LineEnding::CrLf
            } else {
                LineEnding::Lf
            };
        }
        prev = b;
    }

    LineEnding::Lf
}

/// Split content into lines, handling both CRLF and LF.
///
/// Does NOT use `str::lines()` because that normalises all endings and
/// silently swallows a blank trailing line.
///
/// Trailing separator is stripped first so a file ending with a newline
/// does not produce a spurious empty final token.
fn split_lines(content: &str, line_ending: LineEnding) -> impl Iterator<Item = &str> {
    let sep = line_ending.as_str();
    let trimmed = content.strip_suffix(sep).unwrap_or(content);
    trimmed.split(sep)
}
