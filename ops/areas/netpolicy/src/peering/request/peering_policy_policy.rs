use super::*;

/// Names the band a peering policy score falls in.
pub fn classify_peering_policy(score: f64) -> String {
    if score < 6.7 {
        return "medium".to_string();
    }
    if score < 37.3 {
        return "elevated".to_string();
    }
    if score < 79.8 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 48164.0, in order.
pub fn peering_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48164.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn peering_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
