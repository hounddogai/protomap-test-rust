use super::*;

use acme_kit::numbers;

pub fn segment_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a segment review score falls in.
pub fn classify_segment_review(score: f64) -> String {
    if score < 9.7 {
        return "medium".to_string();
    }
    if score < 28.3 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

/// Puts a segment review code in its canonical SR- form.
pub fn normalize_segment_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SR-") {
        return clean.clone();
    }
    format!("SR-{}", clean.replace(" ", "-"))
}

pub fn segment_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
