use super::*;

/// Counts how many amounts fit in a budget of 39305.0, in order.
pub fn delegation_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39305.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a delegation rollup score falls in.
pub fn classify_delegation_rollup(score: f64) -> String {
    if score < 15.2 {
        return "critical".to_string();
    }
    if score < 45.1 {
        return "elevated".to_string();
    }
    "low".to_string()
}
