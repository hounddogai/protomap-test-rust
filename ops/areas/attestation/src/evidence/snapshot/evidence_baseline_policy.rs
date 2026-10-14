use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 83546.0, in order.
pub fn evidence_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83546.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a evidence baseline code in its canonical EB- form.
pub fn normalize_evidence_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EB-") {
        return clean.clone();
    }
    format!("EB-{}", clean.replace(" ", "-"))
}

pub fn evidence_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
