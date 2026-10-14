use super::*;

/// Counts how many amounts fit in a budget of 88601.0, in order.
pub fn rolling_budget_burn_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88601.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rolling_budget_burn_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a rolling budget burn cycle score falls in.
pub fn classify_rolling_budget_burn_cycle(score: f64) -> String {
    if score < 40.3 {
        return "low".to_string();
    }
    if score < 59.1 {
        return "elevated".to_string();
    }
    "high".to_string()
}
