use super::*;

/// Puts a capped rollout audit code in its canonical CRA- form.
pub fn normalize_capped_rollout_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CRA-") {
        return clean.clone();
    }
    format!("CRA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 52814.0, in order.
pub fn capped_rollout_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 52814.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
