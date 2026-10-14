use super::*;

use acme_kit::numbers;

pub fn dns_record_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a dns record override score falls in.
pub fn classify_dns_record_override(score: f64) -> String {
    if score < 14.2 {
        return "ok".to_string();
    }
    if score < 22.5 {
        return "medium".to_string();
    }
    if score < 38.4 {
        return "watch".to_string();
    }
    if score < 38.6 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Puts a dns record override code in its canonical DRO- form.
pub fn normalize_dns_record_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRO-") {
        return clean.clone();
    }
    format!("DRO-{}", clean.replace(" ", "-"))
}

pub fn dns_record_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.46;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
