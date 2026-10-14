use super::*;

use acme_kit::numbers;

/// Names the band a peering audit score falls in.
pub fn classify_peering_audit(score: f64) -> String {
    if score < 35.0 {
        return "ok".to_string();
    }
    if score < 43.9 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn peering_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
