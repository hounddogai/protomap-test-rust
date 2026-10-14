use super::*;

use acme_kit::numbers;

pub fn reservation_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 91382.0, in order.
pub fn reservation_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91382.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a reservation preference score falls in.
pub fn classify_reservation_preference(score: f64) -> String {
    if score < 40.9 {
        return "watch".to_string();
    }
    if score < 42.9 {
        return "low".to_string();
    }
    if score < 68.5 {
        return "high".to_string();
    }
    "medium".to_string()
}
