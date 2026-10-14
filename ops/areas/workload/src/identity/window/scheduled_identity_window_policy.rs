use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 19516.0, in order.
pub fn scheduled_identity_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19516.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a scheduled identity window score falls in.
pub fn classify_scheduled_identity_window(score: f64) -> String {
    if score < 46.7 {
        return "high".to_string();
    }
    if score < 64.4 {
        return "medium".to_string();
    }
    if score < 80.2 {
        return "ok".to_string();
    }
    if score < 84.5 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn scheduled_identity_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
