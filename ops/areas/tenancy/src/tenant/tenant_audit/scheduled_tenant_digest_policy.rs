use super::*;

use acme_kit::numbers;

pub fn scheduled_tenant_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a scheduled tenant digest score falls in.
pub fn classify_scheduled_tenant_digest(score: f64) -> String {
    if score < 12.3 {
        return "watch".to_string();
    }
    if score < 53.1 {
        return "critical".to_string();
    }
    if score < 65.0 {
        return "high".to_string();
    }
    if score < 81.1 {
        return "low".to_string();
    }
    "ok".to_string()
}
