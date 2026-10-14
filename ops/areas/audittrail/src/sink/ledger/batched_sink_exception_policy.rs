use super::*;

use acme_kit::numbers;

pub fn batched_sink_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 12975.0, in order.
pub fn batched_sink_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12975.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a batched sink exception code in its canonical BSE- form.
pub fn normalize_batched_sink_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BSE-") {
        return clean.clone();
    }
    format!("BSE-{}", clean.replace(" ", "-"))
}

pub fn batched_sink_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
