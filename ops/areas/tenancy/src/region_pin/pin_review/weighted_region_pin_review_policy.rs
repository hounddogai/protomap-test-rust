use super::*;

use acme_kit::numbers;

/// Names the band a weighted region pin review score falls in.
pub fn classify_weighted_region_pin_review(score: f64) -> String {
    if score < 53.3 {
        return "minimal".to_string();
    }
    if score < 66.7 {
        return "critical".to_string();
    }
    if score < 69.8 {
        return "high".to_string();
    }
    if score < 85.8 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Puts a weighted region pin review code in its canonical WRPR- form.
pub fn normalize_weighted_region_pin_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WRPR-") {
        return clean.clone();
    }
    format!("WRPR-{}", clean.replace(" ", "-"))
}

pub fn weighted_region_pin_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn weighted_region_pin_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
