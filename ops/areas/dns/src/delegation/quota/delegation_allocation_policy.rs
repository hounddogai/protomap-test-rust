use super::*;

use acme_kit::numbers;

pub fn delegation_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a delegation allocation score falls in.
pub fn classify_delegation_allocation(score: f64) -> String {
    if score < 58.6 {
        return "low".to_string();
    }
    if score < 67.4 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn delegation_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
