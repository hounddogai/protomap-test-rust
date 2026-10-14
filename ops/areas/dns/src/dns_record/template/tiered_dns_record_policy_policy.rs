use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 84935.0, in order.
pub fn tiered_dns_record_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84935.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn tiered_dns_record_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a tiered dns record policy code in its canonical TDRP- form.
pub fn normalize_tiered_dns_record_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDRP-") {
        return clean.clone();
    }
    format!("TDRP-{}", clean.replace(" ", "-"))
}
