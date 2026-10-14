use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 48816.0, in order.
pub fn federation_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48816.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn federation_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn federation_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a federation template code in its canonical FT- form.
pub fn normalize_federation_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FT-") {
        return clean.clone();
    }
    format!("FT-{}", clean.replace(" ", "-"))
}
