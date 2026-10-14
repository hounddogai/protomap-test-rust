use super::*;

/// Names the band a usage threshold score falls in.
pub fn classify_usage_threshold(score: f64) -> String {
    if score < 6.5 {
        return "minimal".to_string();
    }
    if score < 14.0 {
        return "low".to_string();
    }
    if score < 25.6 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 6814.0, in order.
pub fn usage_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6814.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
