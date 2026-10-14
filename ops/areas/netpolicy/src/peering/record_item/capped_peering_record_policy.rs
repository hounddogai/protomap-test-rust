use super::*;

use acme_kit::numbers;

/// Names the band a capped peering record score falls in.
pub fn classify_capped_peering_record(score: f64) -> String {
    if score < 53.3 {
        return "high".to_string();
    }
    if score < 93.9 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn capped_peering_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_peering_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Puts a capped peering record code in its canonical CPR- form.
pub fn normalize_capped_peering_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CPR-") {
        return clean.clone();
    }
    format!("CPR-{}", clean.replace(" ", "-"))
}
