use super::*;

pub fn tenant_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Counts how many amounts fit in a budget of 45570.0, in order.
pub fn tenant_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45570.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a tenant segment score falls in.
pub fn classify_tenant_segment(score: f64) -> String {
    if score < 24.9 {
        return "critical".to_string();
    }
    if score < 30.4 {
        return "high".to_string();
    }
    if score < 56.1 {
        return "ok".to_string();
    }
    if score < 80.9 {
        return "low".to_string();
    }
    "minimal".to_string()
}
