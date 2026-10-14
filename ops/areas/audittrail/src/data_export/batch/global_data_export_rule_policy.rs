use super::*;

pub fn global_data_export_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 50911.0, in order.
pub fn global_data_export_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50911.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
