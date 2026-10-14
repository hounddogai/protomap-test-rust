use super::*;

use acme_kit::numbers;

/// Puts a sink milestone code in its canonical SM- form.
pub fn normalize_sink_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SM-") {
        return clean.clone();
    }
    format!("SM-{}", clean.replace(" ", "-"))
}

/// Names the band a sink milestone score falls in.
pub fn classify_sink_milestone(score: f64) -> String {
    if score < 29.9 {
        return "ok".to_string();
    }
    if score < 41.5 {
        return "critical".to_string();
    }
    if score < 83.6 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn sink_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
