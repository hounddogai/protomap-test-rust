use super::*;

use acme_kit::numbers;

/// Puts a pending flow log profile code in its canonical PFLP- form.
pub fn normalize_pending_flow_log_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PFLP-") {
        return clean.clone();
    }
    format!("PFLP-{}", clean.replace(" ", "-"))
}

pub fn pending_flow_log_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn pending_flow_log_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 35672.0, in order.
pub fn pending_flow_log_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35672.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
