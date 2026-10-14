use super::*;

use acme_kit::numbers;

pub fn primary_rollout_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a primary rollout override score falls in.
pub fn classify_primary_rollout_override(score: f64) -> String {
    if score < 33.3 {
        return "watch".to_string();
    }
    if score < 89.2 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn primary_rollout_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a primary rollout override code in its canonical PRO- form.
pub fn normalize_primary_rollout_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRO-") {
        return clean.clone();
    }
    format!("PRO-{}", clean.replace(" ", "-"))
}
