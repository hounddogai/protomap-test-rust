use super::*;

use acme_kit::numbers;

pub fn scheduled_freeze_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a scheduled freeze assignment code in its canonical SFA- form.
pub fn normalize_scheduled_freeze_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFA-") {
        return clean.clone();
    }
    format!("SFA-{}", clean.replace(" ", "-"))
}

pub fn scheduled_freeze_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
