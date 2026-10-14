use super::*;

use acme_kit::numbers;

/// Names the band a strict rule draft score falls in.
pub fn classify_strict_rule_draft(score: f64) -> String {
    if score < 3.1 {
        return "ok".to_string();
    }
    if score < 20.7 {
        return "high".to_string();
    }
    if score < 43.0 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn strict_rule_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn strict_rule_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 55032.0, in order.
pub fn strict_rule_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55032.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
