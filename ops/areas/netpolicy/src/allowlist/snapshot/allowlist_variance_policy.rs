use super::*;

/// Counts how many amounts fit in a budget of 63057.0, in order.
pub fn allowlist_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63057.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a allowlist variance score falls in.
pub fn classify_allowlist_variance(score: f64) -> String {
    if score < 4.1 {
        return "minimal".to_string();
    }
    if score < 56.3 {
        return "critical".to_string();
    }
    if score < 74.0 {
        return "watch".to_string();
    }
    "high".to_string()
}
