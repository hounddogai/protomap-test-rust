use super::*;

use acme_kit::numbers;

/// Puts a fallback postmortem review code in its canonical FPR- form.
pub fn normalize_fallback_postmortem_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FPR-") {
        return clean.clone();
    }
    format!("FPR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 63026.0, in order.
pub fn fallback_postmortem_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63026.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn fallback_postmortem_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn fallback_postmortem_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
