use super::*;

use acme_kit::numbers;

/// Names the band a adaptive silence assignment score falls in.
pub fn classify_adaptive_silence_assignment(score: f64) -> String {
    if score < 36.1 {
        return "minimal".to_string();
    }
    if score < 74.7 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn adaptive_silence_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Puts a adaptive silence assignment code in its canonical ASA- form.
pub fn normalize_adaptive_silence_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ASA-") {
        return clean.clone();
    }
    format!("ASA-{}", clean.replace(" ", "-"))
}

pub fn adaptive_silence_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.55;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
