use super::*;

use acme_kit::numbers;

pub fn rule_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 10873.0, in order.
pub fn rule_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10873.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rule digest score falls in.
pub fn classify_rule_digest(score: f64) -> String {
    if score < 10.9 {
        return "high".to_string();
    }
    if score < 20.4 {
        return "watch".to_string();
    }
    "low".to_string()
}
