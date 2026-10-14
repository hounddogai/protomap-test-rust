use super::*;

use acme_kit::numbers;

/// Puts a legacy quote group code in its canonical LQG- form.
pub fn normalize_legacy_quote_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LQG-") {
        return clean.clone();
    }
    format!("LQG-{}", clean.replace(" ", "-"))
}

/// Names the band a legacy quote group score falls in.
pub fn classify_legacy_quote_group(score: f64) -> String {
    if score < 8.7 {
        return "high".to_string();
    }
    if score < 17.7 {
        return "critical".to_string();
    }
    if score < 30.9 {
        return "medium".to_string();
    }
    if score < 48.5 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn legacy_quote_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 86372.0, in order.
pub fn legacy_quote_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86372.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
