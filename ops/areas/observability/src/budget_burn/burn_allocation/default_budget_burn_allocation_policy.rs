use super::*;

/// Counts how many amounts fit in a budget of 43796.0, in order.
pub fn default_budget_burn_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43796.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a default budget burn allocation code in its canonical DBBA- form.
pub fn normalize_default_budget_burn_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DBBA-") {
        return clean.clone();
    }
    format!("DBBA-{}", clean.replace(" ", "-"))
}

/// Names the band a default budget burn allocation score falls in.
pub fn classify_default_budget_burn_allocation(score: f64) -> String {
    if score < 93.4 {
        return "low".to_string();
    }
    if score < 96.5 {
        return "critical".to_string();
    }
    if score < 97.9 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
