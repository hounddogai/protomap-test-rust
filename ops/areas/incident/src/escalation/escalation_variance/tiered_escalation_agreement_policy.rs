use super::*;

pub fn tiered_escalation_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 18694.0, in order.
pub fn tiered_escalation_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 18694.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tiered escalation agreement code in its canonical TEA- form.
pub fn normalize_tiered_escalation_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TEA-") {
        return clean.clone();
    }
    format!("TEA-{}", clean.replace(" ", "-"))
}
