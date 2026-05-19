use chrono::{Datelike, NaiveDate, Weekday};

/// Parse a date input string into a `NaiveDate`.
///
/// Accepted formats (D-03 strict whitelist):
/// - `"today"` — returns `today`
/// - `"tomorrow"` — returns `today + 1 day`
/// - weekday names `"monday"` … `"sunday"` — next occurrence (if today is that weekday, returns next week)
/// - `"YYYY-MM-DD"` — ISO date literal
pub fn parse_date_input(s: &str, today: NaiveDate) -> Result<NaiveDate, String> {
    let lower = s.trim().to_lowercase();
    match lower.as_str() {
        "today" => Ok(today),
        "tomorrow" => Ok(today + chrono::Duration::days(1)),
        "monday" => Ok(next_weekday(today, Weekday::Mon)),
        "tuesday" => Ok(next_weekday(today, Weekday::Tue)),
        "wednesday" => Ok(next_weekday(today, Weekday::Wed)),
        "thursday" => Ok(next_weekday(today, Weekday::Thu)),
        "friday" => Ok(next_weekday(today, Weekday::Fri)),
        "saturday" => Ok(next_weekday(today, Weekday::Sat)),
        "sunday" => Ok(next_weekday(today, Weekday::Sun)),
        _ => {
            // Only accept YYYY-MM-DD; reject everything else
            NaiveDate::parse_from_str(lower.as_str(), "%Y-%m-%d").map_err(|_| {
                format!(
                    "invalid date '{}': use today, tomorrow, a weekday name, or YYYY-MM-DD",
                    s
                )
            })
        }
    }
}

/// Return the next occurrence of `target` weekday after `today`.
/// If `today` is already that weekday, return the same day next week.
fn next_weekday(today: NaiveDate, target: Weekday) -> NaiveDate {
    let today_num = today.weekday().num_days_from_monday(); // 0 = Mon, 6 = Sun
    let target_num = target.num_days_from_monday();
    let days_ahead = if target_num > today_num {
        target_num - today_num
    } else {
        7 - (today_num - target_num)
    };
    today + chrono::Duration::days(days_ahead as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    // 2026-04-15 is a Wednesday (weekday 2)
    const TODAY: fn() -> NaiveDate = || d(2026, 4, 15);

    #[test]
    fn test_today() {
        assert_eq!(parse_date_input("today", TODAY()), Ok(TODAY()));
        assert_eq!(parse_date_input("  TODAY  ", TODAY()), Ok(TODAY()));
    }

    #[test]
    fn test_tomorrow() {
        assert_eq!(parse_date_input("tomorrow", TODAY()), Ok(d(2026, 4, 16)));
    }

    #[test]
    fn test_weekday_next_occurrence() {
        // Wednesday is today; next wednesday should be 7 days later
        assert_eq!(parse_date_input("wednesday", TODAY()), Ok(d(2026, 4, 22)));
        // Thursday is tomorrow
        assert_eq!(parse_date_input("thursday", TODAY()), Ok(d(2026, 4, 16)));
        // Monday is 5 days ahead
        assert_eq!(parse_date_input("monday", TODAY()), Ok(d(2026, 4, 20)));
        // Sunday is 4 days ahead
        assert_eq!(parse_date_input("sunday", TODAY()), Ok(d(2026, 4, 19)));
    }

    #[test]
    fn test_iso_date() {
        assert_eq!(parse_date_input("2026-12-25", TODAY()), Ok(d(2026, 12, 25)));
    }

    #[test]
    fn test_invalid_format() {
        assert!(parse_date_input("+3", TODAY()).is_err());
        assert!(parse_date_input("next week", TODAY()).is_err());
        assert!(parse_date_input("25/12/2026", TODAY()).is_err());
        assert!(parse_date_input("2026-13-01", TODAY()).is_err());
    }
}
