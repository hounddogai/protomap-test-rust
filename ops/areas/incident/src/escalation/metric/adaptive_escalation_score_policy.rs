use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 81236.0, in order.
pub fn adaptive_escalation_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 81236.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_escalation_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a adaptive escalation score code in its canonical AES- form.
pub fn normalize_adaptive_escalation_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AES-") {
        return clean.clone();
    }
    format!("AES-{}", clean.replace(" ", "-"))
}
