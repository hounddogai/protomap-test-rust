use super::*;

use acme_kit::numbers;

pub fn tenant_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tenant summary score falls in.
pub fn classify_tenant_summary(score: f64) -> String {
    if score < 11.3 {
        return "elevated".to_string();
    }
    if score < 21.5 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn tenant_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
