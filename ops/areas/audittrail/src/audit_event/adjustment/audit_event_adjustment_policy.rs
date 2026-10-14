use super::*;

use acme_kit::numbers;

/// Names the band a audit event adjustment score falls in.
pub fn classify_audit_event_adjustment(score: f64) -> String {
    if score < 19.3 {
        return "critical".to_string();
    }
    if score < 39.2 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn audit_event_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 45625.0, in order.
pub fn audit_event_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45625.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a audit event adjustment code in its canonical AEA- form.
pub fn normalize_audit_event_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AEA-") {
        return clean.clone();
    }
    format!("AEA-{}", clean.replace(" ", "-"))
}
