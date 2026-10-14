use super::*;

/// Names the band a batched usage manifest score falls in.
pub fn classify_batched_usage_manifest(score: f64) -> String {
    if score < 73.2 {
        return "critical".to_string();
    }
    if score < 83.4 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 10774.0, in order.
pub fn batched_usage_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10774.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
