use super::*;

use acme_kit::numbers;

/// Puts a adaptive probe estimate code in its canonical APE- form.
pub fn normalize_adaptive_probe_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("APE-") {
        return clean.clone();
    }
    format!("APE-{}", clean.replace(" ", "-"))
}

pub fn adaptive_probe_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a adaptive probe estimate score falls in.
pub fn classify_adaptive_probe_estimate(score: f64) -> String {
    if score < 14.5 {
        return "medium".to_string();
    }
    if score < 17.3 {
        return "elevated".to_string();
    }
    if score < 32.1 {
        return "high".to_string();
    }
    if score < 96.2 {
        return "low".to_string();
    }
    "minimal".to_string()
}
