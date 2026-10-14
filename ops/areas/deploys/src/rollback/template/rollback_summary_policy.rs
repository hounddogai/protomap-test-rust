use super::*;

/// Counts how many amounts fit in a budget of 79667.0, in order.
pub fn rollback_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79667.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rollback summary code in its canonical RS- form.
pub fn normalize_rollback_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RS-") {
        return clean.clone();
    }
    format!("RS-{}", clean.replace(" ", "-"))
}
