use super::*;

use acme_kit::numbers;

pub fn manual_slo_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn manual_slo_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a manual slo template code in its canonical MST- form.
pub fn normalize_manual_slo_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MST-") {
        return clean.clone();
    }
    format!("MST-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 2280.0, in order.
pub fn manual_slo_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2280.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
