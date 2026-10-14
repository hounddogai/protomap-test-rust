use super::*;

use acme_kit::numbers;

/// Puts a manual redaction bundle code in its canonical MRB- form.
pub fn normalize_manual_redaction_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRB-") {
        return clean.clone();
    }
    format!("MRB-{}", clean.replace(" ", "-"))
}

pub fn manual_redaction_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a manual redaction bundle score falls in.
pub fn classify_manual_redaction_bundle(score: f64) -> String {
    if score < 21.3 {
        return "minimal".to_string();
    }
    if score < 63.8 {
        return "elevated".to_string();
    }
    if score < 77.0 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn manual_redaction_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
