use super::*;

use acme_kit::numbers;

pub fn trust_domain_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a trust domain agreement score falls in.
pub fn classify_trust_domain_agreement(score: f64) -> String {
    if score < 2.4 {
        return "critical".to_string();
    }
    if score < 34.8 {
        return "ok".to_string();
    }
    if score < 73.1 {
        return "low".to_string();
    }
    if score < 91.4 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn trust_domain_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn trust_domain_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
