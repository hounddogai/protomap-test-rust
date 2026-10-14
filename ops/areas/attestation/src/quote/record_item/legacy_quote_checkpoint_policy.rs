use super::*;

/// Counts how many amounts fit in a budget of 13877.0, in order.
pub fn legacy_quote_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13877.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn legacy_quote_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a legacy quote checkpoint code in its canonical LQC- form.
pub fn normalize_legacy_quote_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LQC-") {
        return clean.clone();
    }
    format!("LQC-{}", clean.replace(" ", "-"))
}
