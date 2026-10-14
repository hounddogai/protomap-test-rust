use super::*;

use acme_kit::numbers;

/// Names the band a weighted measurement group score falls in.
pub fn classify_weighted_measurement_group(score: f64) -> String {
    if score < 26.7 {
        return "minimal".to_string();
    }
    if score < 68.6 {
        return "elevated".to_string();
    }
    if score < 93.7 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn weighted_measurement_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 78738.0, in order.
pub fn weighted_measurement_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 78738.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a weighted measurement group code in its canonical WMG- form.
pub fn normalize_weighted_measurement_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WMG-") {
        return clean.clone();
    }
    format!("WMG-{}", clean.replace(" ", "-"))
}
