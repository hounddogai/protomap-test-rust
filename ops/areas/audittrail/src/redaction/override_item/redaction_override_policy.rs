use super::*;

use acme_kit::numbers;

/// Names the band a redaction override score falls in.
pub fn classify_redaction_override(score: f64) -> String {
    if score < 29.8 {
        return "medium".to_string();
    }
    if score < 46.5 {
        return "elevated".to_string();
    }
    if score < 83.8 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn redaction_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn redaction_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a redaction override code in its canonical RO- form.
pub fn normalize_redaction_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RO-") {
        return clean.clone();
    }
    format!("RO-{}", clean.replace(" ", "-"))
}
