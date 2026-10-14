use super::*;

/// Counts how many amounts fit in a budget of 89495.0, in order.
pub fn pending_query_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89495.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending query bundle score falls in.
pub fn classify_pending_query_bundle(score: f64) -> String {
    if score < 23.4 {
        return "ok".to_string();
    }
    if score < 44.1 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn pending_query_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
