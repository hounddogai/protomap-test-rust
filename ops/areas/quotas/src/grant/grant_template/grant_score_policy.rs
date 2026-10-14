use super::*;

use acme_kit::numbers;

/// Puts a grant score code in its canonical GS- form.
pub fn normalize_grant_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GS-") {
        return clean.clone();
    }
    format!("GS-{}", clean.replace(" ", "-"))
}

pub fn grant_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn grant_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
