use super::*;

use acme_kit::numbers;

pub fn weighted_usage_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 28901.0, in order.
pub fn weighted_usage_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28901.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn weighted_usage_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a weighted usage template code in its canonical WUT- form.
pub fn normalize_weighted_usage_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WUT-") {
        return clean.clone();
    }
    format!("WUT-{}", clean.replace(" ", "-"))
}
