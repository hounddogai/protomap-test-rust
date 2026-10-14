use super::*;

use acme_kit::numbers;

pub fn overage_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn overage_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a overage agreement score falls in.
pub fn classify_overage_agreement(score: f64) -> String {
    if score < 6.0 {
        return "minimal".to_string();
    }
    if score < 8.3 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn overage_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
