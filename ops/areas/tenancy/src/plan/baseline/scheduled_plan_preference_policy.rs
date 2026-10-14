use super::*;

use acme_kit::numbers;

pub fn scheduled_plan_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn scheduled_plan_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a scheduled plan preference score falls in.
pub fn classify_scheduled_plan_preference(score: f64) -> String {
    if score < 1.0 {
        return "ok".to_string();
    }
    if score < 32.8 {
        return "minimal".to_string();
    }
    if score < 45.1 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Puts a scheduled plan preference code in its canonical SPP- form.
pub fn normalize_scheduled_plan_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPP-") {
        return clean.clone();
    }
    format!("SPP-{}", clean.replace(" ", "-"))
}
