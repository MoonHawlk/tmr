//! Minimal, dependency-free UTC calendar math: the current time-of-day and
//! a proleptic-Gregorian civil calendar (year/month/day) from a Unix
//! timestamp, plus the reverse and day-of-week — enough to drive a clock
//! display and a month calendar without pulling in a `chrono`-style crate.
//! Uses Howard Hinnant's `civil_from_days`/`days_from_civil` algorithm (see
//! <http://howardhinnant.github.io/date_algorithms.html>), valid for any
//! date in the proleptic Gregorian calendar.
//!
//! Deliberately UTC-only — a correct local-timezone conversion needs the
//! system's tzdata (DST rules, historical offset changes), which is a much
//! bigger dependency than the sliver of functionality used here (a status
//! bar clock and a month-preview calendar) warrants.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch, per the system clock. `0` if the clock is
/// somehow set before 1970 (never happens in practice; keeps this
/// infallible rather than propagating a `Result` callers can't act on).
pub fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `(hour, minute, second)` within the day, UTC.
pub fn time_of_day(unix_secs: u64) -> (u32, u32, u32) {
    let s = unix_secs % 86_400;
    ((s / 3600) as u32, ((s % 3600) / 60) as u32, (s % 60) as u32)
}

/// A proleptic-Gregorian calendar date (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDate {
    pub year: i32,
    pub month: u32, // 1..=12
    pub day: u32,   // 1..=31
}

impl CivilDate {
    pub fn today() -> Self {
        Self::from_unix_secs(now_unix_secs())
    }

    pub fn from_unix_secs(unix_secs: u64) -> Self {
        let days = (unix_secs / 86_400) as i64;
        let (year, month, day) = civil_from_days(days);
        CivilDate { year, month, day }
    }

    /// `0` = Sunday .. `6` = Saturday.
    pub fn weekday(self) -> u32 {
        let days = days_from_civil(self.year, self.month, self.day);
        // 1970-01-01 (days == 0) was a Thursday (weekday 4).
        (days.rem_euclid(7) + 4).rem_euclid(7) as u32
    }

    pub fn days_in_month(self) -> u32 {
        days_in_month(self.year, self.month)
    }

    pub fn month_name(self) -> &'static str {
        MONTH_NAMES[(self.month - 1) as usize]
    }

    /// The first day (day 1) of this date's month.
    pub fn first_of_month(self) -> Self {
        CivilDate {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    /// This date's month, shifted by `delta` months (may be negative),
    /// wrapping the year as needed. `day` is fixed to `1`, matching
    /// `first_of_month` — callers that only need a month grid (the
    /// Calendar window) never need a specific day here.
    pub fn month_shifted(self, delta: i32) -> Self {
        let zero_based = (self.month as i32 - 1) + delta;
        CivilDate {
            year: self.year + zero_based.div_euclid(12),
            month: (zero_based.rem_euclid(12) + 1) as u32,
            day: 1,
        }
    }
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u32) -> u32 {
    const DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if m == 2 && is_leap_year(y) {
        29
    } else {
        DAYS[(m - 1) as usize]
    }
}

/// Days since 1970-01-01 for a given civil date.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (m as i64 + 9) % 12; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// The inverse of `days_from_civil`: civil date from days since
/// 1970-01-01.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970_01_01() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn known_date_round_trips() {
        let days = days_from_civil(2026, 8, 22);
        assert_eq!(civil_from_days(days), (2026, 8, 22));
    }

    #[test]
    fn epoch_was_a_thursday() {
        let date = CivilDate {
            year: 1970,
            month: 1,
            day: 1,
        };
        assert_eq!(date.weekday(), 4);
    }

    #[test]
    fn leap_year_february_has_29_days() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
    }

    #[test]
    fn time_of_day_splits_seconds_correctly() {
        assert_eq!(time_of_day(3_661), (1, 1, 1));
    }

    #[test]
    fn month_shifted_wraps_forward_across_a_year_boundary() {
        let date = CivilDate {
            year: 2026,
            month: 12,
            day: 22,
        };
        assert_eq!(
            date.month_shifted(1),
            CivilDate {
                year: 2027,
                month: 1,
                day: 1
            }
        );
    }

    #[test]
    fn month_shifted_wraps_backward_across_a_year_boundary() {
        let date = CivilDate {
            year: 2026,
            month: 1,
            day: 15,
        };
        assert_eq!(
            date.month_shifted(-1),
            CivilDate {
                year: 2025,
                month: 12,
                day: 1
            }
        );
    }

    #[test]
    fn month_shifted_handles_multi_year_deltas() {
        let date = CivilDate {
            year: 2026,
            month: 6,
            day: 1,
        };
        assert_eq!(
            date.month_shifted(13),
            CivilDate {
                year: 2027,
                month: 7,
                day: 1
            }
        );
    }

    #[test]
    fn month_name_matches_month_number() {
        let date = CivilDate {
            year: 2026,
            month: 8,
            day: 22,
        };
        assert_eq!(date.month_name(), "August");
    }
}
