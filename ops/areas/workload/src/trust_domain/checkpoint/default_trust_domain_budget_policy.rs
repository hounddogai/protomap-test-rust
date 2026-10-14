use super::*;

use acme_kit::numbers;

/// Puts a default trust domain budget code in its canonical DTDB- form.
pub fn normalize_default_trust_domain_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DTDB-") {
        return clean.clone();
    }
    format!("DTDB-{}", clean.replace(" ", "-"))
}

pub fn default_trust_domain_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 75877.0, in order.
pub fn default_trust_domain_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75877.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
