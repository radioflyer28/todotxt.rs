use std::cmp::Ordering;
use crate::task::Task;

/// The sort order to apply to a `TaskList`.
///
/// All sorts are stable — tasks that compare equal preserve their original order,
/// matching LINQ `OrderBy` behavior from the C# reference implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SortOrder {
    /// `(A)` before `(B)` before unprioritized. None sorts last.
    Priority,
    /// Earliest due date first. Tasks with no due date sort last.
    DueDate,
    /// Case-insensitive raw text comparison.
    Alphabetical,
    /// First `+Project` token alphabetically. Tasks with no project sort last.
    Project,
    /// First `@Context` token alphabetically. Tasks with no context sort last.
    Context,
}

impl SortOrder {
    /// Compare two tasks according to this sort order.
    pub fn compare(&self, a: &Task, b: &Task) -> Ordering {
        match self {
            SortOrder::Priority => {
                match (a.priority, b.priority) {
                    (None, None) => Ordering::Equal,
                    (None, _) => Ordering::Greater,
                    (_, None) => Ordering::Less,
                    (Some(pa), Some(pb)) => pa.cmp(&pb),
                }
            }
            SortOrder::DueDate => {
                match (a.due_date, b.due_date) {
                    (None, None) => Ordering::Equal,
                    (None, _) => Ordering::Greater,
                    (_, None) => Ordering::Less,
                    (Some(da), Some(db)) => da.cmp(&db),
                }
            }
            SortOrder::Alphabetical => {
                a.to_raw().to_ascii_lowercase().cmp(&b.to_raw().to_ascii_lowercase())
            }
            SortOrder::Project => {
                let pa = a.projects.first().map(|s| s.to_ascii_lowercase());
                let pb = b.projects.first().map(|s| s.to_ascii_lowercase());
                match (pa.as_deref(), pb.as_deref()) {
                    (None, None) => Ordering::Equal,
                    (None, _) => Ordering::Greater,
                    (_, None) => Ordering::Less,
                    (Some(pa), Some(pb)) => pa.cmp(pb),
                }
            }
            SortOrder::Context => {
                let ca = a.contexts.first().map(|s| s.to_ascii_lowercase());
                let cb = b.contexts.first().map(|s| s.to_ascii_lowercase());
                match (ca.as_deref(), cb.as_deref()) {
                    (None, None) => Ordering::Equal,
                    (None, _) => Ordering::Greater,
                    (_, None) => Ordering::Less,
                    (Some(ca), Some(cb)) => ca.cmp(cb),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    fn task(raw: &str) -> Task { Task::parse(raw) }

    #[test]
    fn priority_sorted_a_before_b_before_none() {
        let a = task("(A) first");
        let b = task("(B) second");
        let none = task("no priority");
        assert_eq!(SortOrder::Priority.compare(&a, &b), Ordering::Less);
        assert_eq!(SortOrder::Priority.compare(&b, &none), Ordering::Less);
        assert_eq!(SortOrder::Priority.compare(&none, &a), Ordering::Greater);
    }

    #[test]
    fn priority_equal_same_priority() {
        let a = task("(A) first");
        let b = task("(A) second");
        assert_eq!(SortOrder::Priority.compare(&a, &b), Ordering::Equal);
    }

    #[test]
    fn due_date_earliest_first_none_last() {
        let early = task("Old task due:2000-01-01");
        let late = task("Future task due:2099-12-31");
        let none = task("No due date");
        assert_eq!(SortOrder::DueDate.compare(&early, &late), Ordering::Less);
        assert_eq!(SortOrder::DueDate.compare(&none, &early), Ordering::Greater);
    }

    #[test]
    fn alphabetical_case_insensitive() {
        let apple = task("Apple task");
        let banana = task("banana task");
        assert_eq!(SortOrder::Alphabetical.compare(&apple, &banana), Ordering::Less);
    }

    #[test]
    fn project_first_tag_alpha_none_last() {
        let alpha = task("Task +alpha");
        let beta = task("Task +beta");
        let none = task("No project");
        assert_eq!(SortOrder::Project.compare(&alpha, &beta), Ordering::Less);
        assert_eq!(SortOrder::Project.compare(&none, &alpha), Ordering::Greater);
    }

    #[test]
    fn context_first_tag_alpha_none_last() {
        let home = task("Task @home");
        let work = task("Task @work");
        let none = task("No context");
        assert_eq!(SortOrder::Context.compare(&home, &work), Ordering::Less);
        assert_eq!(SortOrder::Context.compare(&none, &home), Ordering::Greater);
    }
}
