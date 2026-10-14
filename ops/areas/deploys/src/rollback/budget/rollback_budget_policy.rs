use super::*;

/// Counts how many amounts fit in a budget of 25386.0, in order.
pub fn rollback_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25386.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rollback budget score falls in.
pub fn classify_rollback_budget(score: f64) -> String {
    if score < 31.6 {
        return "ok".to_string();
    }
    if score < 52.8 {
        return "medium".to_string();
    }
    if score < 73.9 {
        return "critical".to_string();
    }
    if score < 77.2 {
        return "elevated".to_string();
    }
    "low".to_string()
}
