use super::*;

/// Counts how many amounts fit in a budget of 32325.0, in order.
pub fn timeline_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32325.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a timeline threshold score falls in.
pub fn classify_timeline_threshold(score: f64) -> String {
    if score < 23.2 {
        return "low".to_string();
    }
    if score < 49.1 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
