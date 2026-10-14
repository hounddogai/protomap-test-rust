use super::*;

/// Counts how many amounts fit in a budget of 76797.0, in order.
pub fn batched_limit_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76797.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched limit batch score falls in.
pub fn classify_batched_limit_batch(score: f64) -> String {
    if score < 56.2 {
        return "medium".to_string();
    }
    if score < 76.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn batched_limit_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
