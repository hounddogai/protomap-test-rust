use super::*;

use acme_kit::numbers;

/// Names the band a tiered allowlist claim score falls in.
pub fn classify_tiered_allowlist_claim(score: f64) -> String {
    if score < 24.3 {
        return "critical".to_string();
    }
    if score < 74.1 {
        return "ok".to_string();
    }
    if score < 81.7 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn tiered_allowlist_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
