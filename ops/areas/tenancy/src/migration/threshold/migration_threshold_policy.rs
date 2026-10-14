use super::*;

/// Names the band a migration threshold score falls in.
pub fn classify_migration_threshold(score: f64) -> String {
    if score < 42.2 {
        return "elevated".to_string();
    }
    if score < 58.6 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 56953.0, in order.
pub fn migration_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56953.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
