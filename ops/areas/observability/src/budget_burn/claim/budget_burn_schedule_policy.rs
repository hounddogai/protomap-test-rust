use super::*;

use acme_kit::numbers;

/// Puts a budget burn schedule code in its canonical BBS- form.
pub fn normalize_budget_burn_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBS-") {
        return clean.clone();
    }
    format!("BBS-{}", clean.replace(" ", "-"))
}

pub fn budget_burn_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 31960.0, in order.
pub fn budget_burn_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31960.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
