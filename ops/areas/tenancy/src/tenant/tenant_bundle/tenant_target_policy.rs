use super::*;

use acme_kit::numbers;

pub fn tenant_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a tenant target score falls in.
pub fn classify_tenant_target(score: f64) -> String {
    if score < 34.4 {
        return "ok".to_string();
    }
    if score < 70.9 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
