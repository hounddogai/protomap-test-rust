use super::*;

use acme_kit::numbers;

pub fn egress_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a egress bundle score falls in.
pub fn classify_egress_bundle(score: f64) -> String {
    if score < 35.8 {
        return "medium".to_string();
    }
    if score < 36.3 {
        return "elevated".to_string();
    }
    if score < 69.2 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn egress_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a egress bundle code in its canonical EB- form.
pub fn normalize_egress_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EB-") {
        return clean.clone();
    }
    format!("EB-{}", clean.replace(" ", "-"))
}
