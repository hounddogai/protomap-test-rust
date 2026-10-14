use super::*;

use acme_kit::numbers;

pub fn verifier_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a verifier mapping score falls in.
pub fn classify_verifier_mapping(score: f64) -> String {
    if score < 45.8 {
        return "critical".to_string();
    }
    if score < 90.4 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 59354.0, in order.
pub fn verifier_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 59354.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
