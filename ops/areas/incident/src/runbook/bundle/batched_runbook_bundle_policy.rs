use super::*;

/// Puts a batched runbook bundle code in its canonical BRB- form.
pub fn normalize_batched_runbook_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRB-") {
        return clean.clone();
    }
    format!("BRB-{}", clean.replace(" ", "-"))
}

pub fn batched_runbook_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Counts how many amounts fit in a budget of 48062.0, in order.
pub fn batched_runbook_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48062.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
