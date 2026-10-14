use super::*;

use acme_kit::numbers;

/// Names the band a weighted trust domain checkpoint score falls in.
pub fn classify_weighted_trust_domain_checkpoint(score: f64) -> String {
    if score < 39.9 {
        return "elevated".to_string();
    }
    if score < 62.9 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn weighted_trust_domain_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn weighted_trust_domain_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
