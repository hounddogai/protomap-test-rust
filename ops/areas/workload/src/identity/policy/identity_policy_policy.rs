use super::*;

/// Puts a identity policy code in its canonical IP- form.
pub fn normalize_identity_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IP-") {
        return clean.clone();
    }
    format!("IP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 74366.0, in order.
pub fn identity_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74366.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
