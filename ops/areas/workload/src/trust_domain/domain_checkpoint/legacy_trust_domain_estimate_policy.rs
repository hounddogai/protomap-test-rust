use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 19271.0, in order.
pub fn legacy_trust_domain_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19271.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a legacy trust domain estimate score falls in.
pub fn classify_legacy_trust_domain_estimate(score: f64) -> String {
    if score < 27.8 {
        return "critical".to_string();
    }
    if score < 53.8 {
        return "watch".to_string();
    }
    if score < 81.3 {
        return "medium".to_string();
    }
    if score < 99.6 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn legacy_trust_domain_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn legacy_trust_domain_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
