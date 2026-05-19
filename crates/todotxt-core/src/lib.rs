#![deny(warnings)]

pub mod archive;
pub mod error;
pub mod filter;
pub mod portable;
pub mod sort;
pub mod task;
pub mod task_list;
#[cfg(feature = "watching")]
pub mod watcher;

pub use archive::{
    plan_archive_rotation, rotated_archive_path, ArchivePeriod, ArchiveRotationCadence,
    ArchiveRotationDecision,
};
pub use error::TodoError;
pub use filter::{Filter, FilterTerm};
pub use portable::resolve_config_path;
pub use sort::SortOrder;
pub use task::{normalize_append, normalize_line, DueStatus, Task};
pub use task_list::{LineEnding, TaskList};
#[cfg(feature = "watching")]
pub use watcher::FileWatcher;
