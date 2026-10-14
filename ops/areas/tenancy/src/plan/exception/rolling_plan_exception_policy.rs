use super::*;

/// Counts how many amounts fit in a budget of 8164.0, in order.
pub fn rolling_plan_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8164.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rolling plan exception score falls in.
pub fn classify_rolling_plan_exception(score: f64) -> String {
    if score < 19.9 {
        return "medium".to_string();
    }
    if score < 46.7 {
        return "high".to_string();
    }
    if score < 70.1 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
