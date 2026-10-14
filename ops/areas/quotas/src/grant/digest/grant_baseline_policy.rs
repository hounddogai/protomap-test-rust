use super::*;

use acme_kit::numbers;

pub fn grant_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a grant baseline score falls in.
pub fn classify_grant_baseline(score: f64) -> String {
    if score < 8.1 {
        return "ok".to_string();
    }
    if score < 39.6 {
        return "elevated".to_string();
    }
    if score < 47.2 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Puts a grant baseline code in its canonical GB- form.
pub fn normalize_grant_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GB-") {
        return clean.clone();
    }
    format!("GB-{}", clean.replace(" ", "-"))
}

pub fn grant_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
