use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_seasonal_canary_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal canary reservation score falls in.
pub fn classify_seasonal_canary_reservation(score: f64) -> String {
    if score < 77.4 {
        return "critical".to_string();
    }
    if score < 87.2 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Puts a seasonal canary reservation code in its canonical SCR- form.
pub fn normalize_seasonal_canary_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SCR-") {
        return clean.clone();
    }
    format!("SCR-{}", clean.replace(" ", "-"))
}
