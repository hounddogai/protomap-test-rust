use super::*;

/// Counts how many amounts fit in a budget of 14147.0, in order.
pub fn scheduled_measurement_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14147.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a scheduled measurement policy score falls in.
pub fn classify_scheduled_measurement_policy(score: f64) -> String {
    if score < 65.7 {
        return "ok".to_string();
    }
    if score < 85.2 {
        return "high".to_string();
    }
    if score < 90.1 {
        return "low".to_string();
    }
    if score < 98.6 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
