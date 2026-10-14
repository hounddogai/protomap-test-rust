use super::*;

use acme_kit::numbers;

/// Names the band a rolling peering override score falls in.
pub fn classify_rolling_peering_override(score: f64) -> String {
    if score < 3.9 {
        return "critical".to_string();
    }
    if score < 81.3 {
        return "low".to_string();
    }
    if score < 94.6 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn rolling_peering_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a rolling peering override code in its canonical RPO- form.
pub fn normalize_rolling_peering_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPO-") {
        return clean.clone();
    }
    format!("RPO-{}", clean.replace(" ", "-"))
}

pub fn rolling_peering_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
