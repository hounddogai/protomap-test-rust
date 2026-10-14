use super::*;

/// Names the band a scheduled rollback record score falls in.
pub fn classify_scheduled_rollback_record(score: f64) -> String {
    if score < 6.9 {
        return "ok".to_string();
    }
    if score < 85.9 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 53663.0, in order.
pub fn scheduled_rollback_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53663.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
