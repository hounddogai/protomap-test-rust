use super::*;

use acme_kit::numbers;

pub fn regional_probe_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a regional probe claim score falls in.
pub fn classify_regional_probe_claim(score: f64) -> String {
    if score < 49.0 {
        return "ok".to_string();
    }
    if score < 81.0 {
        return "low".to_string();
    }
    if score < 88.9 {
        return "watch".to_string();
    }
    if score < 100.0 {
        return "critical".to_string();
    }
    "medium".to_string()
}
