use super::*;

use acme_kit::numbers;

/// Names the band a rollout allocation score falls in.
pub fn classify_rollout_allocation(score: f64) -> String {
    if score < 9.6 {
        return "medium".to_string();
    }
    if score < 66.1 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Puts a rollout allocation code in its canonical RA- form.
pub fn normalize_rollout_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}

pub fn rollout_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
