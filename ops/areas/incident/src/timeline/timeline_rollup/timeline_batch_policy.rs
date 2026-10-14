use super::*;

pub fn timeline_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a timeline batch score falls in.
pub fn classify_timeline_batch(score: f64) -> String {
    if score < 31.3 {
        return "elevated".to_string();
    }
    if score < 42.9 {
        return "critical".to_string();
    }
    if score < 57.5 {
        return "minimal".to_string();
    }
    if score < 75.2 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 10612.0, in order.
pub fn timeline_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10612.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
