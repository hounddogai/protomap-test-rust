use super::*;

pub fn measurement_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 3131.0, in order.
pub fn measurement_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3131.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a measurement plan score falls in.
pub fn classify_measurement_plan(score: f64) -> String {
    if score < 24.2 {
        return "critical".to_string();
    }
    if score < 39.5 {
        return "medium".to_string();
    }
    if score < 55.4 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
