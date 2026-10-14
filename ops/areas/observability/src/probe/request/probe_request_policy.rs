use super::*;

use acme_kit::numbers;

/// Names the band a probe request score falls in.
pub fn classify_probe_request(score: f64) -> String {
    if score < 18.5 {
        return "low".to_string();
    }
    if score < 37.0 {
        return "ok".to_string();
    }
    if score < 47.9 {
        return "critical".to_string();
    }
    if score < 97.1 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a probe request code in its canonical PR- form.
pub fn normalize_probe_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PR-") {
        return clean.clone();
    }
    format!("PR-{}", clean.replace(" ", "-"))
}

pub fn probe_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.9;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
