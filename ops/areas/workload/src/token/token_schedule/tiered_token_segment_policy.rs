use super::*;

use acme_kit::numbers;

/// Names the band a tiered token segment score falls in.
pub fn classify_tiered_token_segment(score: f64) -> String {
    if score < 17.3 {
        return "low".to_string();
    }
    if score < 30.0 {
        return "watch".to_string();
    }
    if score < 42.1 {
        return "medium".to_string();
    }
    if score < 77.7 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn tiered_token_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a tiered token segment code in its canonical TTS- form.
pub fn normalize_tiered_token_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TTS-") {
        return clean.clone();
    }
    format!("TTS-{}", clean.replace(" ", "-"))
}
