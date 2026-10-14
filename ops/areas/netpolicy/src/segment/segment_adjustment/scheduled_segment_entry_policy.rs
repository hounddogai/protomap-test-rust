use super::*;

use acme_kit::numbers;

pub fn scheduled_segment_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a scheduled segment entry code in its canonical SSE- form.
pub fn normalize_scheduled_segment_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SSE-") {
        return clean.clone();
    }
    format!("SSE-{}", clean.replace(" ", "-"))
}

pub fn scheduled_segment_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn scheduled_segment_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.47;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
