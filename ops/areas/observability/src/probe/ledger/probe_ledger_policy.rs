use super::*;

use acme_kit::numbers;

pub fn probe_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a probe ledger score falls in.
pub fn classify_probe_ledger(score: f64) -> String {
    if score < 25.0 {
        return "minimal".to_string();
    }
    if score < 64.9 {
        return "elevated".to_string();
    }
    if score < 91.1 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn probe_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 68029.0, in order.
pub fn probe_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68029.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
