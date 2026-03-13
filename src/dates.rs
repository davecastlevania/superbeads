use chrono::{Datelike, Duration, NaiveDate, Weekday};

/// Returns all Friday and Saturday departure dates within [from, from + days).
pub fn weekend_dates(from: NaiveDate, days: u32) -> Vec<NaiveDate> {
    let mut result = Vec::new();
    for i in 0..days {
        let date = from + Duration::days(i as i64);
        if matches!(date.weekday(), Weekday::Fri | Weekday::Sat) {
            result.push(date);
        }
    }
    result
}

/// Returns the return date for a weekend departure.
/// Friday -> Monday (+3), Saturday -> Tuesday (+3)
pub fn return_date(departure: NaiveDate) -> NaiveDate {
    departure + Duration::days(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_weekend_dates_only_fri_sat() {
        let from = NaiveDate::from_ymd_opt(2026, 3, 12).unwrap(); // Thursday
        let dates = weekend_dates(from, 14);
        for d in &dates {
            assert!(matches!(d.weekday(), Weekday::Fri | Weekday::Sat),
                "Expected Fri or Sat, got {:?}", d.weekday());
        }
        // 14 days from Thu Mar 12 (days 0..13): Mar 12..25
        // Fri Mar 13 (day 1), Sat Mar 14 (day 2), Fri Mar 20 (day 8), Sat Mar 21 (day 9) = 4
        assert_eq!(dates.len(), 4);
    }

    #[test]
    fn test_return_date() {
        let friday = NaiveDate::from_ymd_opt(2026, 3, 13).unwrap();
        assert_eq!(return_date(friday), NaiveDate::from_ymd_opt(2026, 3, 16).unwrap()); // Monday

        let saturday = NaiveDate::from_ymd_opt(2026, 3, 14).unwrap();
        assert_eq!(return_date(saturday), NaiveDate::from_ymd_opt(2026, 3, 17).unwrap()); // Tuesday
    }
}
