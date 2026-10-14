use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 16157.0, in order.
pub fn query_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16157.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn query_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a query review score falls in.
pub fn classify_query_review(score: f64) -> String {
    if score < 2.6 {
        return "ok".to_string();
    }
    if score < 25.0 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn query_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
