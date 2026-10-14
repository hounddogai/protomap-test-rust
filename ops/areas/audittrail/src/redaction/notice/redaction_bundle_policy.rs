use super::*;

use acme_kit::numbers;

/// Names the band a redaction bundle score falls in.
pub fn classify_redaction_bundle(score: f64) -> String {
    if score < 14.0 {
        return "critical".to_string();
    }
    if score < 86.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a redaction bundle code in its canonical RB- form.
pub fn normalize_redaction_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RB-") {
        return clean.clone();
    }
    format!("RB-{}", clean.replace(" ", "-"))
}

pub fn redaction_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn redaction_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
