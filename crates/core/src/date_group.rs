//! "按时间分组": which Explorer-style section (今天 / 昨天 / 本周 / 本月 / 更早) a
//! modification date belongs to. Pure calendar arithmetic on local civil dates; the platform
//! layer converts Unix times to the user's local date and passes "today" in explicitly.

/// The five date sections, in display order (newest first).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DateBucket {
    Today,
    Yesterday,
    ThisWeek,
    ThisMonth,
    Earlier,
}

/// A calendar date in the user's local time zone (proleptic Gregorian).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CivilDate {
    pub year: i32,
    /// 1..=12
    pub month: u8,
    /// 1..=31
    pub day: u8,
}

impl CivilDate {
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
    pub fn day_number(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let m = i64::from(self.month);
        let d = i64::from(self.day);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = (m + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// Weekday with Monday = 0 … Sunday = 6 (1970-01-01 was a Thursday).
    pub fn weekday_mon0(self) -> u8 {
        (self.day_number() + 3).rem_euclid(7) as u8
    }
}

/// Section of an item dated `item` when the local calendar day is `today`.
///
/// * 今天: the same day, or any later (future) date;
/// * 昨天: the day before;
/// * 本周: earlier in the week that contains `today` (weeks start on Monday);
/// * 本月: earlier in the month that contains `today`;
/// * 更早: everything else.
pub fn date_bucket(item: CivilDate, today: CivilDate) -> DateBucket {
    let diff = today.day_number() - item.day_number();
    if diff <= 0 {
        DateBucket::Today
    } else if diff == 1 {
        DateBucket::Yesterday
    } else if diff <= i64::from(today.weekday_mon0()) {
        DateBucket::ThisWeek
    } else if item.year == today.year && item.month == today.month {
        DateBucket::ThisMonth
    } else {
        DateBucket::Earlier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_numbers_and_weekdays() {
        assert_eq!(CivilDate::new(1970, 1, 1).day_number(), 0);
        assert_eq!(CivilDate::new(1970, 1, 1).weekday_mon0(), 3, "Thursday");
        assert_eq!(CivilDate::new(2000, 3, 1).day_number(), 11_017);
        // 2026-09-07 is a Monday (matches `fileinfo::weekday_is_monday_first`).
        assert_eq!(CivilDate::new(2026, 9, 7).weekday_mon0(), 0);
        assert_eq!(CivilDate::new(2026, 9, 13).weekday_mon0(), 6);
        assert_eq!(CivilDate::new(1969, 12, 31).day_number(), -1);
        assert_eq!(CivilDate::new(1969, 12, 31).weekday_mon0(), 2, "Wednesday");
    }

    #[test]
    fn buckets_follow_explorer_rules() {
        // Wednesday 2026-09-16.
        let today = CivilDate::new(2026, 9, 16);
        assert_eq!(today.weekday_mon0(), 2);
        let b = |y, m, d| date_bucket(CivilDate::new(y, m, d), today);
        assert_eq!(b(2026, 9, 16), DateBucket::Today);
        assert_eq!(
            b(2026, 9, 17),
            DateBucket::Today,
            "future dates read as today"
        );
        assert_eq!(b(2031, 1, 1), DateBucket::Today);
        assert_eq!(b(2026, 9, 15), DateBucket::Yesterday);
        assert_eq!(b(2026, 9, 14), DateBucket::ThisWeek, "Monday of this week");
        assert_eq!(
            b(2026, 9, 13),
            DateBucket::ThisMonth,
            "Sunday belongs to last week"
        );
        assert_eq!(b(2026, 9, 1), DateBucket::ThisMonth);
        assert_eq!(b(2026, 8, 31), DateBucket::Earlier);
        assert_eq!(
            b(2025, 9, 16),
            DateBucket::Earlier,
            "same month, other year"
        );
    }

    #[test]
    fn yesterday_wins_over_this_week_and_week_crosses_the_month() {
        // Monday: yesterday is Sunday, nothing else is "this week".
        let monday = CivilDate::new(2026, 9, 7);
        assert_eq!(
            date_bucket(CivilDate::new(2026, 9, 6), monday),
            DateBucket::Yesterday
        );
        assert_eq!(
            date_bucket(CivilDate::new(2026, 9, 5), monday),
            DateBucket::ThisMonth
        );
        // Thursday 2026-10-01: Monday 09-28 is this week although it is last month.
        let thursday = CivilDate::new(2026, 10, 1);
        assert_eq!(thursday.weekday_mon0(), 3);
        assert_eq!(
            date_bucket(CivilDate::new(2026, 9, 28), thursday),
            DateBucket::ThisWeek
        );
        assert_eq!(
            date_bucket(CivilDate::new(2026, 9, 27), thursday),
            DateBucket::Earlier,
            "last month and before this week"
        );
    }

    /// Items sorted by date descending fall into non-decreasing buckets, so contiguous runs
    /// of one bucket are exactly the sections.
    #[test]
    fn buckets_are_monotone_in_date() {
        for today_day in 0..800i64 {
            let today = civil_from_days(20_000 + today_day);
            let mut prev = DateBucket::Today;
            for back in 0..400i64 {
                let b = date_bucket(civil_from_days(today.day_number() - back), today);
                assert!(b >= prev, "{today:?} -{back}: {b:?} after {prev:?}");
                prev = b;
            }
        }
    }

    fn civil_from_days(z: i64) -> CivilDate {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let date = CivilDate::new((y + i64::from(m <= 2)) as i32, m, d);
        assert_eq!(date.day_number(), z - 719_468);
        date
    }
}
