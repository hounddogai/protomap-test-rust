use super::*;

use acme_kit::numbers;

pub fn limit_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a limit segment score falls in.
pub fn classify_limit_segment(score: f64) -> String {
    if score < 8.2 {
        return "elevated".to_string();
    }
    if score < 34.4 {
        return "watch".to_string();
    }
    if score < 64.1 {
        return "ok".to_string();
    }
    if score < 91.7 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Puts a limit segment code in its canonical LS- form.
pub fn normalize_limit_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LS-") {
        return clean.clone();
    }
    format!("LS-{}", clean.replace(" ", "-"))
}

pub fn limit_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
