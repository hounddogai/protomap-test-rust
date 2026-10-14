use super::*;

use acme_kit::numbers;

pub fn strict_allowlist_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a strict allowlist bundle score falls in.
pub fn classify_strict_allowlist_bundle(score: f64) -> String {
    if score < 14.1 {
        return "watch".to_string();
    }
    if score < 60.7 {
        return "ok".to_string();
    }
    if score < 66.3 {
        return "critical".to_string();
    }
    if score < 81.7 {
        return "minimal".to_string();
    }
    "low".to_string()
}
