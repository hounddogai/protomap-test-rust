use super::*;

use acme_kit::numbers;

/// Names the band a batched rollback batch score falls in.
pub fn classify_batched_rollback_batch(score: f64) -> String {
    if score < 62.0 {
        return "medium".to_string();
    }
    if score < 67.7 {
        return "high".to_string();
    }
    if score < 91.3 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Puts a batched rollback batch code in its canonical BRB- form.
pub fn normalize_batched_rollback_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRB-") {
        return clean.clone();
    }
    format!("BRB-{}", clean.replace(" ", "-"))
}

pub fn batched_rollback_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
