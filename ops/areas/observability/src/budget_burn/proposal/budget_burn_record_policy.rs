use super::*;

/// Counts how many amounts fit in a budget of 46212.0, in order.
pub fn budget_burn_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46212.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a budget burn record score falls in.
pub fn classify_budget_burn_record(score: f64) -> String {
    if score < 31.8 {
        return "medium".to_string();
    }
    if score < 79.3 {
        return "low".to_string();
    }
    if score < 85.5 {
        return "high".to_string();
    }
    "ok".to_string()
}
