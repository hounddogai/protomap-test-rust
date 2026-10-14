use super::*;

/// Counts how many amounts fit in a budget of 72505.0, in order.
pub fn primary_canary_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72505.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a primary canary tier code in its canonical PCT- form.
pub fn normalize_primary_canary_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PCT-") {
        return clean.clone();
    }
    format!("PCT-{}", clean.replace(" ", "-"))
}
