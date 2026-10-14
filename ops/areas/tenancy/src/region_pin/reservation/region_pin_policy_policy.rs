use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 90779.0, in order.
pub fn region_pin_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90779.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a region pin policy score falls in.
pub fn classify_region_pin_policy(score: f64) -> String {
    if score < 38.5 {
        return "high".to_string();
    }
    if score < 49.0 {
        return "watch".to_string();
    }
    if score < 83.6 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Puts a region pin policy code in its canonical RPP- form.
pub fn normalize_region_pin_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPP-") {
        return clean.clone();
    }
    format!("RPP-{}", clean.replace(" ", "-"))
}

pub fn region_pin_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
