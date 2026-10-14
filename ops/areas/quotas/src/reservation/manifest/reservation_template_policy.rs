use super::*;

use acme_kit::numbers;

pub fn reservation_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 8512.0, in order.
pub fn reservation_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8512.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a reservation template score falls in.
pub fn classify_reservation_template(score: f64) -> String {
    if score < 56.8 {
        return "medium".to_string();
    }
    if score < 71.7 {
        return "ok".to_string();
    }
    if score < 80.9 {
        return "high".to_string();
    }
    "watch".to_string()
}
