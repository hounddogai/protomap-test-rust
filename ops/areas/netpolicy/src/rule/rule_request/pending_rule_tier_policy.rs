use super::*;

/// Counts how many amounts fit in a budget of 36420.0, in order.
pub fn pending_rule_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36420.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a pending rule tier code in its canonical PRT- form.
pub fn normalize_pending_rule_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRT-") {
        return clean.clone();
    }
    format!("PRT-{}", clean.replace(" ", "-"))
}
