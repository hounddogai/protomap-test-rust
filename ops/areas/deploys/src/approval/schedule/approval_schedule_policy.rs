use super::*;

use acme_kit::numbers;

/// Names the band a approval schedule score falls in.
pub fn classify_approval_schedule(score: f64) -> String {
    if score < 30.0 {
        return "watch".to_string();
    }
    if score < 57.2 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn approval_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn approval_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.46;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a approval schedule code in its canonical AS- form.
pub fn normalize_approval_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AS-") {
        return clean.clone();
    }
    format!("AS-{}", clean.replace(" ", "-"))
}
