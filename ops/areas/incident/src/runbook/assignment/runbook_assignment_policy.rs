use super::*;

/// Counts how many amounts fit in a budget of 41258.0, in order.
pub fn runbook_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41258.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn runbook_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a runbook assignment code in its canonical RA- form.
pub fn normalize_runbook_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}
