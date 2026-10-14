use super::*;

use acme_kit::numbers;

/// Puts a global silence allocation code in its canonical GSA- form.
pub fn normalize_global_silence_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GSA-") {
        return clean.clone();
    }
    format!("GSA-{}", clean.replace(" ", "-"))
}

pub fn global_silence_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn global_silence_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 7082.0, in order.
pub fn global_silence_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7082.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
