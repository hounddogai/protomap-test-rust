use super::*;

use acme_kit::numbers;

pub fn grant_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.4;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a grant preference code in its canonical GP- form.
pub fn normalize_grant_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GP-") {
        return clean.clone();
    }
    format!("GP-{}", clean.replace(" ", "-"))
}

pub fn grant_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a grant preference score falls in.
pub fn classify_grant_preference(score: f64) -> String {
    if score < 35.7 {
        return "critical".to_string();
    }
    if score < 38.7 {
        return "elevated".to_string();
    }
    if score < 70.3 {
        return "medium".to_string();
    }
    if score < 71.3 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
