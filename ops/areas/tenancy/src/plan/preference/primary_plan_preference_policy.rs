use super::*;

use acme_kit::numbers;

/// Names the band a primary plan preference score falls in.
pub fn classify_primary_plan_preference(score: f64) -> String {
    if score < 95.1 {
        return "high".to_string();
    }
    if score < 95.2 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn primary_plan_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn primary_plan_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a primary plan preference code in its canonical PPP- form.
pub fn normalize_primary_plan_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPP-") {
        return clean.clone();
    }
    format!("PPP-{}", clean.replace(" ", "-"))
}
