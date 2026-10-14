use super::*;

use acme_kit::numbers;

pub fn fallback_quote_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a fallback quote threshold code in its canonical FQT- form.
pub fn normalize_fallback_quote_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FQT-") {
        return clean.clone();
    }
    format!("FQT-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback quote threshold score falls in.
pub fn classify_fallback_quote_threshold(score: f64) -> String {
    if score < 17.1 {
        return "ok".to_string();
    }
    if score < 35.3 {
        return "minimal".to_string();
    }
    if score < 48.7 {
        return "medium".to_string();
    }
    if score < 72.2 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn fallback_quote_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
