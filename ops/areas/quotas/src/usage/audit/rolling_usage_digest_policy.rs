use super::*;

use acme_kit::numbers;

/// Puts a rolling usage digest code in its canonical RUD- form.
pub fn normalize_rolling_usage_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RUD-") {
        return clean.clone();
    }
    format!("RUD-{}", clean.replace(" ", "-"))
}

pub fn rolling_usage_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn rolling_usage_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
