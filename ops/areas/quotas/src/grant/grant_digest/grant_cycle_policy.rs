use super::*;

/// Counts how many amounts fit in a budget of 44104.0, in order.
pub fn grant_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44104.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a grant cycle code in its canonical GC- form.
pub fn normalize_grant_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GC-") {
        return clean.clone();
    }
    format!("GC-{}", clean.replace(" ", "-"))
}
