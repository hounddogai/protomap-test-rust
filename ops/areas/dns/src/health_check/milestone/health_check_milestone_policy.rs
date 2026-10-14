use super::*;

/// Puts a health check milestone code in its canonical HCM- form.
pub fn normalize_health_check_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCM-") {
        return clean.clone();
    }
    format!("HCM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 39808.0, in order.
pub fn health_check_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39808.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
