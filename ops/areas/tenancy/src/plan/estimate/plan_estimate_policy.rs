use super::*;

use acme_kit::numbers;

pub fn plan_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 69055.0, in order.
pub fn plan_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69055.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a plan estimate score falls in.
pub fn classify_plan_estimate(score: f64) -> String {
    if score < 16.6 {
        return "watch".to_string();
    }
    if score < 41.9 {
        return "low".to_string();
    }
    if score < 52.9 {
        return "elevated".to_string();
    }
    if score < 82.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
