use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 49526.0, in order.
pub fn primary_egress_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49526.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_egress_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a primary egress score code in its canonical PES- form.
pub fn normalize_primary_egress_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PES-") {
        return clean.clone();
    }
    format!("PES-{}", clean.replace(" ", "-"))
}

pub fn primary_egress_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
