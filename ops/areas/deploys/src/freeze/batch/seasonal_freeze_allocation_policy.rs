use super::*;

/// Names the band a seasonal freeze allocation score falls in.
pub fn classify_seasonal_freeze_allocation(score: f64) -> String {
    if score < 75.5 {
        return "low".to_string();
    }
    if score < 84.4 {
        return "elevated".to_string();
    }
    if score < 88.6 {
        return "high".to_string();
    }
    if score < 94.8 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 46145.0, in order.
pub fn seasonal_freeze_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46145.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
