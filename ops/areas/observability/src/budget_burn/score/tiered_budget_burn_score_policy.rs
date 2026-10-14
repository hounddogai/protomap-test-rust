use super::*;

/// Counts how many amounts fit in a budget of 95353.0, in order.
pub fn tiered_budget_burn_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95353.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tiered budget burn score code in its canonical TBBS- form.
pub fn normalize_tiered_budget_burn_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TBBS-") {
        return clean.clone();
    }
    format!("TBBS-{}", clean.replace(" ", "-"))
}

pub fn tiered_budget_burn_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
