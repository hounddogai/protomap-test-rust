use super::*;

/// Puts a pending runbook estimate code in its canonical PRE- form.
pub fn normalize_pending_runbook_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRE-") {
        return clean.clone();
    }
    format!("PRE-{}", clean.replace(" ", "-"))
}

pub fn pending_runbook_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 42709.0, in order.
pub fn pending_runbook_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 42709.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
