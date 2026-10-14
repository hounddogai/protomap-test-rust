use super::*;

use acme_kit::numbers;

/// Puts a pending query audit code in its canonical PQA- form.
pub fn normalize_pending_query_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PQA-") {
        return clean.clone();
    }
    format!("PQA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 41735.0, in order.
pub fn pending_query_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41735.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_query_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
