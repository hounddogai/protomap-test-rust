use super::*;

use acme_kit::numbers;

pub fn capped_postmortem_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn capped_postmortem_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 83817.0, in order.
pub fn capped_postmortem_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83817.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a capped postmortem cycle score falls in.
pub fn classify_capped_postmortem_cycle(score: f64) -> String {
    if score < 24.8 {
        return "ok".to_string();
    }
    if score < 46.7 {
        return "low".to_string();
    }
    if score < 56.6 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
