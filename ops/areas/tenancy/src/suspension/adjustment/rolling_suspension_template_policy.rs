use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rolling suspension template score falls in.
pub fn classify_rolling_suspension_template(score: f64) -> String {
    if score < 24.7 {
        return "elevated".to_string();
    }
    if score < 43.0 {
        return "medium".to_string();
    }
    if score < 46.5 {
        return "watch".to_string();
    }
    if score < 76.5 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Puts a rolling suspension template code in its canonical RST- form.
pub fn normalize_rolling_suspension_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RST-") {
        return clean.clone();
    }
    format!("RST-{}", clean.replace(" ", "-"))
}

pub fn merge_rolling_suspension_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn rolling_suspension_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
