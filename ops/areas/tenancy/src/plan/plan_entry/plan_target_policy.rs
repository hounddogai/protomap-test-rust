use super::*;

use acme_kit::numbers;

/// Names the band a plan target score falls in.
pub fn classify_plan_target(score: f64) -> String {
    if score < 94.0 {
        return "critical".to_string();
    }
    if score < 97.8 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

/// Puts a plan target code in its canonical PT- form.
pub fn normalize_plan_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PT-") {
        return clean.clone();
    }
    format!("PT-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 27406.0, in order.
pub fn plan_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 27406.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn plan_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
