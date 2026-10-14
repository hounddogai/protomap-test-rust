use super::*;

/// Names the band a weighted federation estimate score falls in.
pub fn classify_weighted_federation_estimate(score: f64) -> String {
    if score < 75.8 {
        return "critical".to_string();
    }
    if score < 87.0 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 63352.0, in order.
pub fn weighted_federation_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63352.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
