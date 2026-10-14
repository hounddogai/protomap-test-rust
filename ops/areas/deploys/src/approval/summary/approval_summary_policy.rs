use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 91532.0, in order.
pub fn approval_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91532.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn approval_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn approval_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a approval summary code in its canonical AS- form.
pub fn normalize_approval_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AS-") {
        return clean.clone();
    }
    format!("AS-{}", clean.replace(" ", "-"))
}
