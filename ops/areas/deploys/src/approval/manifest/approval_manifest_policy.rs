use super::*;

/// Names the band a approval manifest score falls in.
pub fn classify_approval_manifest(score: f64) -> String {
    if score < 42.2 {
        return "critical".to_string();
    }
    if score < 71.2 {
        return "low".to_string();
    }
    if score < 94.7 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 70122.0, in order.
pub fn approval_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70122.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
