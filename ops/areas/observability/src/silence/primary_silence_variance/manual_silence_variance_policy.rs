use super::*;

use acme_kit::numbers;

/// Names the band a manual silence variance score falls in.
pub fn classify_manual_silence_variance(score: f64) -> String {
    if score < 24.9 {
        return "medium".to_string();
    }
    if score < 25.6 {
        return "watch".to_string();
    }
    if score < 35.4 {
        return "high".to_string();
    }
    if score < 37.0 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Puts a manual silence variance code in its canonical MSV- form.
pub fn normalize_manual_silence_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MSV-") {
        return clean.clone();
    }
    format!("MSV-{}", clean.replace(" ", "-"))
}

pub fn manual_silence_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn manual_silence_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
