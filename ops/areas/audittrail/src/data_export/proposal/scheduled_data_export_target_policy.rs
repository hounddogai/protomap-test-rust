use super::*;

pub fn scheduled_data_export_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Counts how many amounts fit in a budget of 18098.0, in order.
pub fn scheduled_data_export_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 18098.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
