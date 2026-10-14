use super::*;

use acme_kit::numbers;

/// Names the band a grant summary score falls in.
pub fn classify_grant_summary(score: f64) -> String {
    if score < 6.1 {
        return "low".to_string();
    }
    if score < 35.7 {
        return "minimal".to_string();
    }
    if score < 99.7 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 47271.0, in order.
pub fn grant_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47271.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn grant_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.34;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
