use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 17298.0, in order.
pub fn scheduled_suspension_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17298.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a scheduled suspension override score falls in.
pub fn classify_scheduled_suspension_override(score: f64) -> String {
    if score < 18.8 {
        return "medium".to_string();
    }
    if score < 28.3 {
        return "critical".to_string();
    }
    if score < 49.5 {
        return "minimal".to_string();
    }
    if score < 75.4 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn scheduled_suspension_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
