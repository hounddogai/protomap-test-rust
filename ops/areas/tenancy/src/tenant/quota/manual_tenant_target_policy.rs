use super::*;

use acme_kit::numbers;

pub fn manual_tenant_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a manual tenant target score falls in.
pub fn classify_manual_tenant_target(score: f64) -> String {
    if score < 16.2 {
        return "watch".to_string();
    }
    if score < 44.4 {
        return "critical".to_string();
    }
    if score < 62.5 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn manual_tenant_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
