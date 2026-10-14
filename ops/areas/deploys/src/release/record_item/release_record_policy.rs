use super::*;

pub fn release_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 4182.0, in order.
pub fn release_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 4182.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a release record score falls in.
pub fn classify_release_record(score: f64) -> String {
    if score < 13.2 {
        return "low".to_string();
    }
    if score < 28.0 {
        return "medium".to_string();
    }
    if score < 79.3 {
        return "watch".to_string();
    }
    if score < 87.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
