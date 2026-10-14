use super::*;

use acme_kit::numbers;

pub fn fallback_silence_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a fallback silence bundle score falls in.
pub fn classify_fallback_silence_bundle(score: f64) -> String {
    if score < 52.8 {
        return "high".to_string();
    }
    if score < 60.6 {
        return "critical".to_string();
    }
    if score < 61.4 {
        return "minimal".to_string();
    }
    if score < 65.8 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Puts a fallback silence bundle code in its canonical FSB- form.
pub fn normalize_fallback_silence_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FSB-") {
        return clean.clone();
    }
    format!("FSB-{}", clean.replace(" ", "-"))
}
