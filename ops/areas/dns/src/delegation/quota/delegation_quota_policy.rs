use super::*;

use acme_kit::numbers;

pub fn delegation_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a delegation quota code in its canonical DQ- form.
pub fn normalize_delegation_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DQ-") {
        return clean.clone();
    }
    format!("DQ-{}", clean.replace(" ", "-"))
}

pub fn delegation_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.23;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 82956.0, in order.
pub fn delegation_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 82956.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
