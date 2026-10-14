use super::*;

/// Counts how many amounts fit in a budget of 98281.0, in order.
pub fn release_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98281.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a release entry code in its canonical RE- form.
pub fn normalize_release_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RE-") {
        return clean.clone();
    }
    format!("RE-{}", clean.replace(" ", "-"))
}
