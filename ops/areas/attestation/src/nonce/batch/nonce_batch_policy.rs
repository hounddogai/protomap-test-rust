use super::*;

use acme_kit::numbers;

pub fn nonce_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 95750.0, in order.
pub fn nonce_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95750.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a nonce batch score falls in.
pub fn classify_nonce_batch(score: f64) -> String {
    if score < 11.9 {
        return "low".to_string();
    }
    if score < 29.2 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
