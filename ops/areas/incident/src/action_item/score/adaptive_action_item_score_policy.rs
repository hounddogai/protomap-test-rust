use super::*;

use acme_kit::numbers;

pub fn adaptive_action_item_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn adaptive_action_item_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a adaptive action item score code in its canonical AAIS- form.
pub fn normalize_adaptive_action_item_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AAIS-") {
        return clean.clone();
    }
    format!("AAIS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 38431.0, in order.
pub fn adaptive_action_item_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38431.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
