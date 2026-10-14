use super::*;

use acme_kit::numbers;

/// Puts a retention variance code in its canonical RV- form.
pub fn normalize_retention_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RV-") {
        return clean.clone();
    }
    format!("RV-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 18762.0, in order.
pub fn retention_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 18762.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn retention_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.17;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a retention variance score falls in.
pub fn classify_retention_variance(score: f64) -> String {
    if score < 8.9 {
        return "elevated".to_string();
    }
    if score < 20.6 {
        return "high".to_string();
    }
    "minimal".to_string()
}
