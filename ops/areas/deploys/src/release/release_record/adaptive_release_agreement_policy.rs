use super::*;

use acme_kit::numbers;

/// Names the band a adaptive release agreement score falls in.
pub fn classify_adaptive_release_agreement(score: f64) -> String {
    if score < 18.1 {
        return "watch".to_string();
    }
    if score < 24.4 {
        return "ok".to_string();
    }
    if score < 52.4 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 15838.0, in order.
pub fn adaptive_release_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15838.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_release_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
