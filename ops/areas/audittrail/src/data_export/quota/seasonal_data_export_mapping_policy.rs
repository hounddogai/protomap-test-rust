use super::*;

use acme_kit::numbers;

/// Names the band a seasonal data export mapping score falls in.
pub fn classify_seasonal_data_export_mapping(score: f64) -> String {
    if score < 15.0 {
        return "medium".to_string();
    }
    if score < 85.5 {
        return "minimal".to_string();
    }
    if score < 87.4 {
        return "watch".to_string();
    }
    if score < 87.5 {
        return "critical".to_string();
    }
    "low".to_string()
}

/// Puts a seasonal data export mapping code in its canonical SDEM- form.
pub fn normalize_seasonal_data_export_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SDEM-") {
        return clean.clone();
    }
    format!("SDEM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 95367.0, in order.
pub fn seasonal_data_export_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95367.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn seasonal_data_export_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
