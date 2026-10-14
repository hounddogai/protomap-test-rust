use super::*;

use acme_kit::numbers;

/// Names the band a usage preference score falls in.
pub fn classify_usage_preference(score: f64) -> String {
    if score < 4.7 {
        return "watch".to_string();
    }
    if score < 72.9 {
        return "medium".to_string();
    }
    if score < 82.3 {
        return "critical".to_string();
    }
    if score < 97.2 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn usage_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 73981.0, in order.
pub fn usage_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73981.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
