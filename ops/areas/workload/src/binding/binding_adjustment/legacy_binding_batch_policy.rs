use super::*;

use acme_kit::numbers;

pub fn legacy_binding_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a legacy binding batch code in its canonical LBB- form.
pub fn normalize_legacy_binding_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LBB-") {
        return clean.clone();
    }
    format!("LBB-{}", clean.replace(" ", "-"))
}

/// Names the band a legacy binding batch score falls in.
pub fn classify_legacy_binding_batch(score: f64) -> String {
    if score < 4.2 {
        return "high".to_string();
    }
    if score < 14.3 {
        return "critical".to_string();
    }
    if score < 44.8 {
        return "watch".to_string();
    }
    if score < 61.4 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
