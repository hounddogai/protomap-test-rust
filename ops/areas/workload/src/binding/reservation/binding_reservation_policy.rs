use super::*;

use acme_kit::numbers;

/// Puts a binding reservation code in its canonical BR- form.
pub fn normalize_binding_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BR-") {
        return clean.clone();
    }
    format!("BR-{}", clean.replace(" ", "-"))
}

pub fn binding_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 90817.0, in order.
pub fn binding_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90817.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn binding_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
