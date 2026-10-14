use super::*;

use acme_kit::numbers;

pub fn segment_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 90823.0, in order.
pub fn segment_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90823.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a segment allocation code in its canonical SA- form.
pub fn normalize_segment_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SA-") {
        return clean.clone();
    }
    format!("SA-{}", clean.replace(" ", "-"))
}

pub fn segment_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
