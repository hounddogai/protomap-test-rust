use super::*;

use acme_kit::numbers;

pub fn egress_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a egress group code in its canonical EG- form.
pub fn normalize_egress_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EG-") {
        return clean.clone();
    }
    format!("EG-{}", clean.replace(" ", "-"))
}

pub fn egress_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a egress group score falls in.
pub fn classify_egress_group(score: f64) -> String {
    if score < 39.0 {
        return "low".to_string();
    }
    if score < 72.8 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
