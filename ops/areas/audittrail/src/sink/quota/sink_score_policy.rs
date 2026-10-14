use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a sink score code in its canonical SS- form.
pub fn normalize_sink_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SS-") {
        return clean.clone();
    }
    format!("SS-{}", clean.replace(" ", "-"))
}

pub fn merge_sink_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn sink_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a sink score score falls in.
pub fn classify_sink_score(score: f64) -> String {
    if score < 1.4 {
        return "high".to_string();
    }
    if score < 43.6 {
        return "ok".to_string();
    }
    if score < 62.4 {
        return "medium".to_string();
    }
    if score < 68.9 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
