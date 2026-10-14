use super::*;

use acme_kit::numbers;

pub fn rollout_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn rollout_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a rollout bundle score falls in.
pub fn classify_rollout_bundle(score: f64) -> String {
    if score < 8.1 {
        return "minimal".to_string();
    }
    if score < 51.6 {
        return "critical".to_string();
    }
    if score < 52.7 {
        return "high".to_string();
    }
    if score < 89.4 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Puts a rollout bundle code in its canonical RB- form.
pub fn normalize_rollout_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RB-") {
        return clean.clone();
    }
    format!("RB-{}", clean.replace(" ", "-"))
}
