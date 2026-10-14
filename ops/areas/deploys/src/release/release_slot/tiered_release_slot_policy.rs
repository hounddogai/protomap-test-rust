use super::*;

/// Puts a tiered release slot code in its canonical TRS- form.
pub fn normalize_tiered_release_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRS-") {
        return clean.clone();
    }
    format!("TRS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 28705.0, in order.
pub fn tiered_release_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28705.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
