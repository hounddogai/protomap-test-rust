use super::*;

/// Counts how many amounts fit in a budget of 5662.0, in order.
pub fn manual_policy_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5662.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a manual policy assignment code in its canonical MPA- form.
pub fn normalize_manual_policy_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MPA-") {
        return clean.clone();
    }
    format!("MPA-{}", clean.replace(" ", "-"))
}
