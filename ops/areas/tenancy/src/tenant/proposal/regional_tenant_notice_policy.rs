use super::*;

use acme_kit::numbers;

pub fn regional_tenant_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a regional tenant notice code in its canonical RTN- form.
pub fn normalize_regional_tenant_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTN-") {
        return clean.clone();
    }
    format!("RTN-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 45718.0, in order.
pub fn regional_tenant_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45718.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
