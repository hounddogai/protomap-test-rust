use super::*;

use acme_kit::numbers;

/// Names the band a verifier manifest score falls in.
pub fn classify_verifier_manifest(score: f64) -> String {
    if score < 39.3 {
        return "ok".to_string();
    }
    if score < 59.5 {
        return "high".to_string();
    }
    if score < 79.4 {
        return "minimal".to_string();
    }
    if score < 97.2 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn verifier_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn verifier_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a verifier manifest code in its canonical VM- form.
pub fn normalize_verifier_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VM-") {
        return clean.clone();
    }
    format!("VM-{}", clean.replace(" ", "-"))
}
