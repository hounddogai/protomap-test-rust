use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 84855.0, in order.
pub fn segment_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84855.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn segment_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a segment score score falls in.
pub fn classify_segment_score(score: f64) -> String {
    if score < 39.6 {
        return "ok".to_string();
    }
    if score < 72.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Puts a segment score code in its canonical SS- form.
pub fn normalize_segment_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SS-") {
        return clean.clone();
    }
    format!("SS-{}", clean.replace(" ", "-"))
}
