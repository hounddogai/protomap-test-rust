use super::*;

use acme_kit::numbers;

pub fn health_check_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.13;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a health check preference code in its canonical HCP- form.
pub fn normalize_health_check_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCP-") {
        return clean.clone();
    }
    format!("HCP-{}", clean.replace(" ", "-"))
}

/// Names the band a health check preference score falls in.
pub fn classify_health_check_preference(score: f64) -> String {
    if score < 39.8 {
        return "medium".to_string();
    }
    if score < 56.1 {
        return "high".to_string();
    }
    if score < 69.8 {
        return "ok".to_string();
    }
    if score < 72.5 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 30455.0, in order.
pub fn health_check_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30455.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
