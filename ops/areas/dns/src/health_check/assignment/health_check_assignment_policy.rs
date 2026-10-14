use super::*;

/// Counts how many amounts fit in a budget of 53236.0, in order.
pub fn health_check_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53236.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a health check assignment score falls in.
pub fn classify_health_check_assignment(score: f64) -> String {
    if score < 28.0 {
        return "low".to_string();
    }
    if score < 46.8 {
        return "watch".to_string();
    }
    if score < 53.5 {
        return "high".to_string();
    }
    "elevated".to_string()
}
