use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_peering_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a peering score score falls in.
pub fn classify_peering_score(score: f64) -> String {
    if score < 35.4 {
        return "minimal".to_string();
    }
    if score < 59.9 {
        return "low".to_string();
    }
    if score < 74.5 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Puts a peering score code in its canonical PS- form.
pub fn normalize_peering_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PS-") {
        return clean.clone();
    }
    format!("PS-{}", clean.replace(" ", "-"))
}

pub fn peering_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
