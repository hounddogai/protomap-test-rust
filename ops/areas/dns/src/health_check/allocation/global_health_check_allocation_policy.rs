use super::*;

/// Puts a global health check allocation code in its canonical GHCA- form.
pub fn normalize_global_health_check_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GHCA-") {
        return clean.clone();
    }
    format!("GHCA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 53705.0, in order.
pub fn global_health_check_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53705.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
