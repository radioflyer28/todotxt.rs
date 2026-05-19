use chrono::{DateTime, Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Archive rotation cadence for `done.txt`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveRotationCadence {
    #[default]
    Monthly,
}

/// A deterministic archive period bucket derived from a date and cadence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchivePeriod {
    year: i32,
    ordinal: u32,
}

impl ArchivePeriod {
    pub fn from_date(date: NaiveDate, cadence: ArchiveRotationCadence) -> Self {
        match cadence {
            ArchiveRotationCadence::Monthly => Self {
                year: date.year(),
                ordinal: date.month(),
            },
        }
    }

    pub fn suffix(self, cadence: ArchiveRotationCadence) -> String {
        match cadence {
            ArchiveRotationCadence::Monthly => {
                format!("{:04}-{:02}", self.year, self.ordinal)
            }
        }
    }
}

/// Rotation decision for the current archive write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveRotationDecision {
    pub current_period: ArchivePeriod,
    pub existing_period: Option<ArchivePeriod>,
    pub rotated_path: Option<PathBuf>,
}

/// Convert a filesystem timestamp into an archive period.
pub fn archive_period_from_system_time(
    system_time: SystemTime,
    cadence: ArchiveRotationCadence,
) -> ArchivePeriod {
    let date = DateTime::<Local>::from(system_time).date_naive();
    ArchivePeriod::from_date(date, cadence)
}

/// Build the deterministic rotated archive path for a prior active archive period.
pub fn rotated_archive_path(
    done_path: &Path,
    period: ArchivePeriod,
    cadence: ArchiveRotationCadence,
) -> PathBuf {
    let parent = done_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = done_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "done".to_string());
    let suffix = period.suffix(cadence);
    match done_path
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
    {
        Some(ext) if !ext.is_empty() => parent.join(format!("{stem}-{suffix}.{ext}")),
        _ => parent.join(format!("{stem}-{suffix}")),
    }
}

/// Decide whether an existing active archive should rotate before a new archive write.
pub fn plan_archive_rotation(
    done_path: &Path,
    cadence: ArchiveRotationCadence,
    archive_date: NaiveDate,
    existing_modified: Option<SystemTime>,
    existing_has_content: bool,
) -> ArchiveRotationDecision {
    let current_period = ArchivePeriod::from_date(archive_date, cadence);
    let existing_period =
        existing_modified.map(|time| archive_period_from_system_time(time, cadence));
    let rotated_path = if existing_has_content {
        existing_period
            .filter(|period| *period != current_period)
            .map(|period| rotated_archive_path(done_path, period, cadence))
    } else {
        None
    };

    ArchiveRotationDecision {
        current_period,
        existing_period,
        rotated_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use std::time::{Duration, UNIX_EPOCH};

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn noon_system_time(year: i32, month: u32, day: u32) -> SystemTime {
        let naive = NaiveDateTime::new(
            date(year, month, day),
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );
        let seconds = naive.and_utc().timestamp();
        UNIX_EPOCH + Duration::from_secs(seconds as u64)
    }

    #[test]
    fn archive_rotation_period_monthly() {
        let period = ArchivePeriod::from_date(date(2026, 5, 19), ArchiveRotationCadence::Monthly);
        assert_eq!(period.suffix(ArchiveRotationCadence::Monthly), "2026-05");
    }

    #[test]
    fn rotated_archive_path_uses_period_suffix() {
        let path = PathBuf::from("/tmp/done.txt");
        let period = ArchivePeriod::from_date(date(2026, 1, 31), ArchiveRotationCadence::Monthly);
        assert_eq!(
            rotated_archive_path(&path, period, ArchiveRotationCadence::Monthly),
            PathBuf::from("/tmp/done-2026-01.txt")
        );
    }

    #[test]
    fn plan_archive_rotation_skips_same_period() {
        let decision = plan_archive_rotation(
            Path::new("/tmp/done.txt"),
            ArchiveRotationCadence::Monthly,
            date(2026, 5, 19),
            Some(noon_system_time(2026, 5, 3)),
            true,
        );
        assert!(
            decision.rotated_path.is_none(),
            "same-month archive should not rotate"
        );
    }

    #[test]
    fn plan_archive_rotation_rotates_prior_period_with_content() {
        let decision = plan_archive_rotation(
            Path::new("/tmp/done.txt"),
            ArchiveRotationCadence::Monthly,
            date(2026, 5, 19),
            Some(noon_system_time(2026, 4, 30)),
            true,
        );
        assert_eq!(
            decision.rotated_path,
            Some(PathBuf::from("/tmp/done-2026-04.txt"))
        );
    }

    #[test]
    fn plan_archive_rotation_skips_empty_existing_archive() {
        let decision = plan_archive_rotation(
            Path::new("/tmp/done.txt"),
            ArchiveRotationCadence::Monthly,
            date(2026, 5, 19),
            Some(noon_system_time(2026, 4, 30)),
            false,
        );
        assert!(
            decision.rotated_path.is_none(),
            "empty active archive should not produce an empty rotated file"
        );
    }
}
