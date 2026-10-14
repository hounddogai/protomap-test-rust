use super::*;

use acme_kit::numbers;

pub fn plan_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn plan_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a plan adjustment code in its canonical PA- form.
pub fn normalize_plan_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PA-") {
        return clean.clone();
    }
    format!("PA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 347.0, in order.
pub fn plan_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 347.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
