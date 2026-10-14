use super::*;

/// Puts a pending release rule code in its canonical PRR- form.
pub fn normalize_pending_release_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRR-") {
        return clean.clone();
    }
    format!("PRR-{}", clean.replace(" ", "-"))
}

pub fn pending_release_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 13766.0, in order.
pub fn pending_release_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13766.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
