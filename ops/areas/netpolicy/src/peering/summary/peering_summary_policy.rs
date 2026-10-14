use super::*;

use acme_kit::numbers;

/// Names the band a peering summary score falls in.
pub fn classify_peering_summary(score: f64) -> String {
    if score < 10.1 {
        return "minimal".to_string();
    }
    if score < 83.0 {
        return "low".to_string();
    }
    if score < 91.4 {
        return "watch".to_string();
    }
    if score < 98.5 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Puts a peering summary code in its canonical PS- form.
pub fn normalize_peering_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PS-") {
        return clean.clone();
    }
    format!("PS-{}", clean.replace(" ", "-"))
}

pub fn peering_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
