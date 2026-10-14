use super::*;

/// Counts how many amounts fit in a budget of 39376.0, in order.
pub fn allowlist_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39376.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a allowlist window code in its canonical AW- form.
pub fn normalize_allowlist_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AW-") {
        return clean.clone();
    }
    format!("AW-{}", clean.replace(" ", "-"))
}
