use super::*;

/// Counts how many amounts fit in a budget of 68368.0, in order.
pub fn strict_limit_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68368.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a strict limit assignment score falls in.
pub fn classify_strict_limit_assignment(score: f64) -> String {
    if score < 51.3 {
        return "elevated".to_string();
    }
    if score < 71.3 {
        return "minimal".to_string();
    }
    if score < 74.7 {
        return "medium".to_string();
    }
    "critical".to_string()
}
