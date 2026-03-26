use iou_core::sla::{SlaCalculator, SlaConfig};
use chrono::{DateTime, Utc, Duration, Weekday, Datelike};
use std::collections::HashSet;

#[tokio::test]
async fn test_calculate_deadline_adds_business_hours() {
    let calculator = SlaCalculator::new();

    // Monday 10 AM + 8 hours = Tuesday 10 AM (next business day)
    let start = DateTime::parse_from_rfc3339("2024-01-08T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let deadline = calculator.calculate_deadline(start, 8);

    // Should be approximately 8 hours later
    let diff = deadline.signed_duration_since(start);
    assert!(diff.num_hours() >= 8);
}

#[tokio::test]
async fn test_calculate_deadline_skips_saturday() {
    let calculator = SlaCalculator::new();

    // Friday 10 AM + 24 hours
    // Friday 10 AM -> Monday 10 AM (skips Sat, Sun)
    let start = DateTime::parse_from_rfc3339("2024-01-05T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let deadline = calculator.calculate_deadline(start, 24);

    // Should be Monday (not weekend)
    assert_eq!(deadline.weekday(), Weekday::Mon);
}

#[tokio::test]
async fn test_calculate_deadline_skips_sunday() {
    let calculator = SlaCalculator::new();

    // Saturday 10 AM + 8 hours
    // Saturday -> Monday (skips Sun)
    let start = DateTime::parse_from_rfc3339("2024-01-06T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let deadline = calculator.calculate_deadline(start, 8);

    // Should not be weekend
    assert!(!calculator.is_weekend(deadline));
}

#[tokio::test]
async fn test_is_overdue_true() {
    let calculator = SlaCalculator::new();

    let past = Utc::now() - Duration::hours(1);
    assert!(calculator.is_overdue(past));
}

#[tokio::test]
async fn test_is_overdue_false() {
    let calculator = SlaCalculator::new();

    let future = Utc::now() + Duration::hours(1);
    assert!(!calculator.is_overdue(future));
}

#[tokio::test]
async fn test_hours_until_deadline_future() {
    let calculator = SlaCalculator::new();

    let deadline = Utc::now() + Duration::hours(48);
    let hours = calculator.hours_until_deadline(deadline);

    assert!(hours > 0);
    assert!(hours <= 48); // May be less due to weekends
}

#[tokio::test]
async fn test_hours_until_deadline_past() {
    let calculator = SlaCalculator::new();

    let deadline = Utc::now() - Duration::hours(24);
    let hours = calculator.hours_until_deadline(deadline);

    assert!(hours < 0);
    assert!(hours >= -48); // May be more negative due to weekends
}

#[tokio::test]
async fn test_is_weekend_saturday() {
    let calculator = SlaCalculator::new();

    let saturday = DateTime::parse_from_rfc3339("2024-01-06T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    assert!(calculator.is_weekend(saturday));
}

#[tokio::test]
async fn test_is_weekend_sunday() {
    let calculator = SlaCalculator::new();

    let sunday = DateTime::parse_from_rfc3339("2024-01-07T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    assert!(calculator.is_weekend(sunday));
}

#[tokio::test]
async fn test_is_weekend_monday_false() {
    let calculator = SlaCalculator::new();

    let monday = DateTime::parse_from_rfc3339("2024-01-08T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    assert!(!calculator.is_weekend(monday));
}

#[tokio::test]
async fn test_custom_weekend_days() {
    let config = SlaConfig {
        weekend_days: vec![Weekday::Fri, Weekday::Sat],
        ..Default::default()
    };
    let calculator = SlaCalculator::with_config(config);

    let friday = DateTime::parse_from_rfc3339("2024-01-05T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    assert!(calculator.is_weekend(friday));

    let sunday = DateTime::parse_from_rfc3339("2024-01-07T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    assert!(!calculator.is_weekend(sunday));
}

#[tokio::test]
async fn test_sla_config_dutch_weekend() {
    // Dutch weekend is Saturday/Sunday (default)
    let calculator = SlaCalculator::new();

    let saturday = DateTime::parse_from_rfc3339("2024-01-06T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let sunday = DateTime::parse_from_rfc3339("2024-01-07T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    assert!(calculator.is_weekend(saturday));
    assert!(calculator.is_weekend(sunday));
}
