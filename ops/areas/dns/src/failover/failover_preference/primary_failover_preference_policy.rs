use super::*;

/// Puts a primary failover preference code in its canonical PFP- form.
pub fn normalize_primary_failover_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PFP-") {
        return clean.clone();
    }
    format!("PFP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 94612.0, in order.
pub fn primary_failover_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94612.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
