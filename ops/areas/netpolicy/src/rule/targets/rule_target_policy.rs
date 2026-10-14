use super::*;

use acme_kit::numbers;

/// Puts a rule target code in its canonical RT- form.
pub fn normalize_rule_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RT-") {
        return clean.clone();
    }
    format!("RT-{}", clean.replace(" ", "-"))
}

/// Names the band a rule target score falls in.
pub fn classify_rule_target(score: f64) -> String {
    if score < 84.4 {
        return "minimal".to_string();
    }
    if score < 98.6 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn rule_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 18050.0, in order.
pub fn rule_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 18050.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
