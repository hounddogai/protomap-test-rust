use super::*;

use acme_kit::numbers;

pub fn weighted_migration_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a weighted migration claim score falls in.
pub fn classify_weighted_migration_claim(score: f64) -> String {
    if score < 42.9 {
        return "ok".to_string();
    }
    if score < 86.8 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 56934.0, in order.
pub fn weighted_migration_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56934.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
