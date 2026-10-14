use super::*;

use acme_kit::numbers;

pub fn limit_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a limit entry score falls in.
pub fn classify_limit_entry(score: f64) -> String {
    if score < 14.8 {
        return "medium".to_string();
    }
    if score < 51.4 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn limit_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
