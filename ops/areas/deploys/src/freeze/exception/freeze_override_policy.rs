use super::*;

use acme_kit::numbers;

pub fn freeze_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a freeze override code in its canonical FO- form.
pub fn normalize_freeze_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FO-") {
        return clean.clone();
    }
    format!("FO-{}", clean.replace(" ", "-"))
}

pub fn freeze_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.12;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a freeze override score falls in.
pub fn classify_freeze_override(score: f64) -> String {
    if score < 6.7 {
        return "high".to_string();
    }
    if score < 12.8 {
        return "low".to_string();
    }
    if score < 27.0 {
        return "elevated".to_string();
    }
    if score < 68.8 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
