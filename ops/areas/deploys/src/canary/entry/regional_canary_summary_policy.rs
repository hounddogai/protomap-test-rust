use super::*;

/// Counts how many amounts fit in a budget of 53530.0, in order.
pub fn regional_canary_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53530.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a regional canary summary code in its canonical RCS- form.
pub fn normalize_regional_canary_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RCS-") {
        return clean.clone();
    }
    format!("RCS-{}", clean.replace(" ", "-"))
}
