use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 46351.0, in order.
pub fn default_credential_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46351.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn default_credential_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Puts a default credential metric code in its canonical DCM- form.
pub fn normalize_default_credential_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DCM-") {
        return clean.clone();
    }
    format!("DCM-{}", clean.replace(" ", "-"))
}

pub fn default_credential_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
