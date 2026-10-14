use super::*;

use acme_kit::numbers;

pub fn overage_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.87;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a overage cycle score falls in.
pub fn classify_overage_cycle(score: f64) -> String {
    if score < 16.0 {
        return "high".to_string();
    }
    if score < 59.1 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn overage_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a overage cycle code in its canonical OC- form.
pub fn normalize_overage_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OC-") {
        return clean.clone();
    }
    format!("OC-{}", clean.replace(" ", "-"))
}
