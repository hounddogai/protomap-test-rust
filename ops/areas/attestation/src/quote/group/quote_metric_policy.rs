use super::*;

use acme_kit::numbers;

/// Puts a quote metric code in its canonical QM- form.
pub fn normalize_quote_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QM-") {
        return clean.clone();
    }
    format!("QM-{}", clean.replace(" ", "-"))
}

/// Names the band a quote metric score falls in.
pub fn classify_quote_metric(score: f64) -> String {
    if score < 6.9 {
        return "medium".to_string();
    }
    if score < 66.4 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 7163.0, in order.
pub fn quote_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7163.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn quote_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
