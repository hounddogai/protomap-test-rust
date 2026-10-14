use super::*;

use acme_kit::numbers;

pub fn pending_region_pin_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a pending region pin metric code in its canonical PRPM- form.
pub fn normalize_pending_region_pin_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRPM-") {
        return clean.clone();
    }
    format!("PRPM-{}", clean.replace(" ", "-"))
}

/// Names the band a pending region pin metric score falls in.
pub fn classify_pending_region_pin_metric(score: f64) -> String {
    if score < 6.8 {
        return "low".to_string();
    }
    if score < 52.2 {
        return "minimal".to_string();
    }
    if score < 93.2 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
