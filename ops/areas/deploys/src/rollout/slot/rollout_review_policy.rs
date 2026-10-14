use super::*;

use acme_kit::numbers;

/// Puts a rollout review code in its canonical RR- form.
pub fn normalize_rollout_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RR-") {
        return clean.clone();
    }
    format!("RR-{}", clean.replace(" ", "-"))
}

pub fn rollout_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rollout review score falls in.
pub fn classify_rollout_review(score: f64) -> String {
    if score < 25.9 {
        return "ok".to_string();
    }
    if score < 66.3 {
        return "low".to_string();
    }
    if score < 81.7 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
