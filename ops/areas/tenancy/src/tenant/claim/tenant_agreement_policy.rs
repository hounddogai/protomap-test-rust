use super::*;

use acme_kit::numbers;

/// Names the band a tenant agreement score falls in.
pub fn classify_tenant_agreement(score: f64) -> String {
    if score < 34.7 {
        return "watch".to_string();
    }
    if score < 71.8 {
        return "critical".to_string();
    }
    if score < 75.2 {
        return "elevated".to_string();
    }
    if score < 94.9 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn tenant_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
