use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 16048.0, in order.
pub fn approval_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16048.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn approval_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a approval rule score falls in.
pub fn classify_approval_rule(score: f64) -> String {
    if score < 3.6 {
        return "medium".to_string();
    }
    if score < 23.4 {
        return "ok".to_string();
    }
    if score < 42.9 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
