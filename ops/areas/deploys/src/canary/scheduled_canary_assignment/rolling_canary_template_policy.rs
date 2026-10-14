use super::*;

use acme_kit::numbers;

pub fn rolling_canary_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn rolling_canary_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a rolling canary template score falls in.
pub fn classify_rolling_canary_template(score: f64) -> String {
    if score < 17.3 {
        return "watch".to_string();
    }
    if score < 51.6 {
        return "elevated".to_string();
    }
    if score < 72.3 {
        return "medium".to_string();
    }
    if score < 93.8 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a rolling canary template code in its canonical RCT- form.
pub fn normalize_rolling_canary_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RCT-") {
        return clean.clone();
    }
    format!("RCT-{}", clean.replace(" ", "-"))
}
