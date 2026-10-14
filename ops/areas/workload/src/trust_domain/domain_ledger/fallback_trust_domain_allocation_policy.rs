use super::*;

use acme_kit::numbers;

pub fn fallback_trust_domain_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_trust_domain_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn fallback_trust_domain_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a fallback trust domain allocation score falls in.
pub fn classify_fallback_trust_domain_allocation(score: f64) -> String {
    if score < 1.1 {
        return "minimal".to_string();
    }
    if score < 75.3 {
        return "critical".to_string();
    }
    if score < 81.4 {
        return "watch".to_string();
    }
    if score < 99.7 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
