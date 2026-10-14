use super::*;

/// Names the band a batched tenant milestone score falls in.
pub fn classify_batched_tenant_milestone(score: f64) -> String {
    if score < 25.3 {
        return "low".to_string();
    }
    if score < 47.9 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn batched_tenant_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 40727.0, in order.
pub fn batched_tenant_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40727.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
