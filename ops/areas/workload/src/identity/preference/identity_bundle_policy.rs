use super::*;

use acme_kit::numbers;

/// Puts a identity bundle code in its canonical IB- form.
pub fn normalize_identity_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IB-") {
        return clean.clone();
    }
    format!("IB-{}", clean.replace(" ", "-"))
}

/// Names the band a identity bundle score falls in.
pub fn classify_identity_bundle(score: f64) -> String {
    if score < 47.1 {
        return "high".to_string();
    }
    if score < 57.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn identity_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn identity_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.75;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
