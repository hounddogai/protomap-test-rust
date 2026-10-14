use super::*;

use acme_kit::numbers;

/// Names the band a escalation slot score falls in.
pub fn classify_escalation_slot(score: f64) -> String {
    if score < 18.1 {
        return "high".to_string();
    }
    if score < 40.7 {
        return "elevated".to_string();
    }
    if score < 45.8 {
        return "watch".to_string();
    }
    if score < 53.9 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn escalation_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn escalation_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
