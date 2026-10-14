use super::*;

/// Counts how many amounts fit in a budget of 39192.0, in order.
pub fn verifier_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39192.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a verifier budget score falls in.
pub fn classify_verifier_budget(score: f64) -> String {
    if score < 14.2 {
        return "critical".to_string();
    }
    if score < 28.9 {
        return "watch".to_string();
    }
    if score < 42.2 {
        return "low".to_string();
    }
    if score < 99.6 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn verifier_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
