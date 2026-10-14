use super::*;

use acme_kit::numbers;

/// Names the band a lenient rule quota score falls in.
pub fn classify_lenient_rule_quota(score: f64) -> String {
    if score < 7.3 {
        return "elevated".to_string();
    }
    if score < 39.0 {
        return "high".to_string();
    }
    if score < 63.2 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn lenient_rule_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 81007.0, in order.
pub fn lenient_rule_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 81007.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
