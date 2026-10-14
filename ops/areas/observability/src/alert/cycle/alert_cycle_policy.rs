use super::*;

use acme_kit::numbers;

pub fn alert_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 80582.0, in order.
pub fn alert_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80582.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a alert cycle score falls in.
pub fn classify_alert_cycle(score: f64) -> String {
    if score < 10.2 {
        return "medium".to_string();
    }
    if score < 69.7 {
        return "watch".to_string();
    }
    "critical".to_string()
}
