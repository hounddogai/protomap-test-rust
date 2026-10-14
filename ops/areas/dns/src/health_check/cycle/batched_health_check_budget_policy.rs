use super::*;

/// Counts how many amounts fit in a budget of 9552.0, in order.
pub fn batched_health_check_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9552.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a batched health check budget code in its canonical BHCB- form.
pub fn normalize_batched_health_check_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BHCB-") {
        return clean.clone();
    }
    format!("BHCB-{}", clean.replace(" ", "-"))
}
