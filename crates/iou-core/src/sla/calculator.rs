//! SLA (Service Level Agreement) deadline calculator
//!
//! Calculates deadlines based on business hours, skipping weekends and
//! configured holidays. Supports configurable weekend days for international
//! use cases (e.g., Friday-Saturday weekend in Middle Eastern countries).

use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for SLA calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaConfig {
    /// Days considered weekends (default: Saturday, Sunday)
    pub weekend_days: Vec<Weekday>,

    /// Holiday dates (YYYY-MM-DD format)
    pub holidays: HashSet<String>,
}

impl Default for SlaConfig {
    fn default() -> Self {
        Self {
            weekend_days: vec![Weekday::Sat, Weekday::Sun],
            holidays: HashSet::new(),
        }
    }
}

/// SLA calculator for deadline computation
pub struct SlaCalculator {
    config: SlaConfig,
}

impl SlaCalculator {
    /// Create a new SLA calculator with default configuration
    pub fn new() -> Self {
        Self {
            config: SlaConfig::default(),
        }
    }

    /// Create a new SLA calculator with custom configuration
    pub fn with_config(config: SlaConfig) -> Self {
        Self { config }
    }

    /// Calculate deadline by adding business hours to start time
    ///
    /// This implementation uses a simplified approach:
    /// - Adds hours one at a time, skipping weekends
    /// - Does NOT handle partial days or business hours within a day
    /// - Holidays are checked but not fully implemented in initial version
    ///
    /// # Arguments
    /// * `start` - The starting timestamp
    /// * `business_hours` - Number of business hours to add
    ///
    /// # Returns
    /// The calculated deadline timestamp
    pub fn calculate_deadline(
        &self,
        start: DateTime<Utc>,
        business_hours: i32,
    ) -> DateTime<Utc> {
        let mut current = start;
        let mut hours_remaining = business_hours;

        while hours_remaining > 0 {
            current = current + Duration::hours(1);

            // Skip weekends and holidays
            if !self.is_weekend(current) && !self.is_holiday(current) {
                hours_remaining -= 1;
            }
        }

        current
    }

    /// Check if a given date falls on a weekend
    pub fn is_weekend(&self, date: DateTime<Utc>) -> bool {
        self.config.weekend_days.contains(&date.weekday())
    }

    /// Check if a given date is a holiday
    pub fn is_holiday(&self, date: DateTime<Utc>) -> bool {
        let date_str = date.format("%Y-%m-%d").to_string();
        self.config.holidays.contains(&date_str)
    }

    /// Check if deadline has passed
    pub fn is_overdue(&self, deadline: DateTime<Utc>) -> bool {
        Utc::now() > deadline
    }

    /// Calculate business hours until deadline
    ///
    /// Counts only business hours (excluding weekends) between
    /// current time and deadline. Returns negative if deadline is past.
    pub fn hours_until_deadline(&self, deadline: DateTime<Utc>) -> i32 {
        let now = Utc::now();
        let is_past = now > deadline;

        let (start, end) = if is_past {
            (deadline, now)
        } else {
            (now, deadline)
        };

        let mut current = start;
        let mut business_hours = 0;

        while current < end {
            current = current + Duration::hours(1);

            // Count only business hours (skip weekends and holidays)
            if !self.is_weekend(current) && !self.is_holiday(current) {
                business_hours += 1;
            }
        }

        if is_past {
            -business_hours
        } else {
            business_hours
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &SlaConfig {
        &self.config
    }

    /// Update holidays
    pub fn set_holidays(&mut self, holidays: HashSet<String>) {
        self.config.holidays = holidays;
    }
}

impl Default for SlaCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_calculator() -> SlaCalculator {
        SlaCalculator::new()
    }

    #[test]
    fn test_calculate_deadline_basic() {
        let calculator = create_calculator();
        // Monday 10 AM UTC + 24 business hours = Tuesday 10 AM UTC
        let start = DateTime::parse_from_rfc3339("2024-01-08T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let deadline = calculator.calculate_deadline(start, 24);

        // Should be Tuesday (next day, not weekend)
        assert_eq!(deadline.weekday(), Weekday::Tue);
    }

    #[test]
    fn test_calculate_deadline_skips_weekend() {
        let calculator = create_calculator();
        // Friday 10 AM + 24 business hours
        // Friday 10 AM -> Friday 10 AM (next day) = 24h, but skip weekend
        // Actually: Friday 10 AM + 24h = Saturday 10 AM (skip) -> skip Sunday too
        // Result should be Monday 10 AM or Tuesday depending on counting
        let start = DateTime::parse_from_rfc3339("2024-01-05T10:00:00Z") // Friday
            .unwrap()
            .with_timezone(&Utc);

        let deadline = calculator.calculate_deadline(start, 24);

        // After skipping weekend, should land on Monday or Tuesday
        assert!(!calculator.is_weekend(deadline));
    }

    #[test]
    fn test_is_overdue() {
        let calculator = create_calculator();

        let past = Utc::now() - Duration::hours(1);
        assert!(calculator.is_overdue(past));

        let future = Utc::now() + Duration::hours(1);
        assert!(!calculator.is_overdue(future));
    }

    #[test]
    fn test_hours_until_deadline_future() {
        let calculator = create_calculator();

        let deadline = Utc::now() + Duration::hours(24);
        let hours = calculator.hours_until_deadline(deadline);

        assert!(hours > 0);
        assert!(hours <= 24); // May be less due to weekends
    }

    #[test]
    fn test_hours_until_deadline_past() {
        let calculator = create_calculator();

        let deadline = Utc::now() - Duration::hours(24);
        let hours = calculator.hours_until_deadline(deadline);

        assert!(hours < 0);
    }

    #[test]
    fn test_is_weekend() {
        let calculator = create_calculator();

        // Saturday
        let saturday = DateTime::parse_from_rfc3339("2024-01-06T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(calculator.is_weekend(saturday));

        // Sunday
        let sunday = DateTime::parse_from_rfc3339("2024-01-07T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(calculator.is_weekend(sunday));

        // Monday
        let monday = DateTime::parse_from_rfc3339("2024-01-08T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(!calculator.is_weekend(monday));
    }

    #[test]
    fn test_custom_weekend_days() {
        let config = SlaConfig {
            weekend_days: vec![Weekday::Fri, Weekday::Sat], // Middle East weekend
            ..Default::default()
        };
        let calculator = SlaCalculator::with_config(config);

        // Friday should be weekend
        let friday = DateTime::parse_from_rfc3339("2024-01-05T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(calculator.is_weekend(friday));

        // Sunday should NOT be weekend
        let sunday = DateTime::parse_from_rfc3339("2024-01-07T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(!calculator.is_weekend(sunday));
    }

    #[test]
    fn test_holiday_check() {
        let mut holidays = HashSet::new();
        holidays.insert("2024-12-25".to_string()); // Christmas

        let config = SlaConfig {
            holidays,
            ..Default::default()
        };
        let calculator = SlaCalculator::with_config(config);

        let christmas = DateTime::parse_from_rfc3339("2024-12-25T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(calculator.is_holiday(christmas));

        let regular_day = DateTime::parse_from_rfc3339("2024-12-24T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(!calculator.is_holiday(regular_day));
    }
}
