use super::*;

use acme_kit::numbers;

/// Puts a primary evidence entry code in its canonical PEE- form.
pub fn normalize_primary_evidence_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PEE-") {
        return clean.clone();
    }
    format!("PEE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 65536.0, in order.
pub fn primary_evidence_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 65536.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_evidence_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.81;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn primary_evidence_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
