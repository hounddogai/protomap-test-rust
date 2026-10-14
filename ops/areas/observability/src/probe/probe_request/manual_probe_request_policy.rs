use super::*;

use acme_kit::numbers;

pub fn manual_probe_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn manual_probe_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a manual probe request score falls in.
pub fn classify_manual_probe_request(score: f64) -> String {
    if score < 9.4 {
        return "medium".to_string();
    }
    if score < 25.8 {
        return "minimal".to_string();
    }
    if score < 45.0 {
        return "low".to_string();
    }
    "high".to_string()
}

/// Puts a manual probe request code in its canonical MPR- form.
pub fn normalize_manual_probe_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MPR-") {
        return clean.clone();
    }
    format!("MPR-{}", clean.replace(" ", "-"))
}
