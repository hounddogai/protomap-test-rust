use super::*;

pub fn token_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a token metric score falls in.
pub fn classify_token_metric(score: f64) -> String {
    if score < 5.4 {
        return "watch".to_string();
    }
    if score < 7.5 {
        return "high".to_string();
    }
    if score < 13.4 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 69031.0, in order.
pub fn token_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69031.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
