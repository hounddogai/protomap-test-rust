use super::*;

use acme_kit::numbers;

pub fn allowlist_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 41540.0, in order.
pub fn allowlist_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41540.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a allowlist profile score falls in.
pub fn classify_allowlist_profile(score: f64) -> String {
    if score < 31.2 {
        return "medium".to_string();
    }
    if score < 53.7 {
        return "low".to_string();
    }
    if score < 72.3 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
