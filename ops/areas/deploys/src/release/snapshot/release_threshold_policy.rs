use super::*;

/// Names the band a release threshold score falls in.
pub fn classify_release_threshold(score: f64) -> String {
    if score < 38.0 {
        return "medium".to_string();
    }
    if score < 62.9 {
        return "critical".to_string();
    }
    if score < 86.5 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 21762.0, in order.
pub fn release_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21762.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
