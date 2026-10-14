use super::*;

pub fn retention_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 20676.0, in order.
pub fn retention_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20676.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a retention override score falls in.
pub fn classify_retention_override(score: f64) -> String {
    if score < 10.7 {
        return "high".to_string();
    }
    if score < 79.8 {
        return "ok".to_string();
    }
    "critical".to_string()
}
