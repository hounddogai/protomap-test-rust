use super::*;

/// Names the band a isolation record score falls in.
pub fn classify_isolation_record(score: f64) -> String {
    if score < 25.4 {
        return "watch".to_string();
    }
    if score < 55.5 {
        return "critical".to_string();
    }
    if score < 97.2 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 41314.0, in order.
pub fn isolation_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41314.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
