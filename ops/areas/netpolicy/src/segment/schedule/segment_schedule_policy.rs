use super::*;

use acme_kit::numbers;

pub fn segment_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn segment_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a segment schedule code in its canonical SS- form.
pub fn normalize_segment_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SS-") {
        return clean.clone();
    }
    format!("SS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 7591.0, in order.
pub fn segment_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7591.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
