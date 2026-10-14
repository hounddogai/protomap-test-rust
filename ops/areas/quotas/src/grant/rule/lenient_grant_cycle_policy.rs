use super::*;

use acme_kit::numbers;

pub fn lenient_grant_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a lenient grant cycle code in its canonical LGC- form.
pub fn normalize_lenient_grant_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LGC-") {
        return clean.clone();
    }
    format!("LGC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 95675.0, in order.
pub fn lenient_grant_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95675.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_grant_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
