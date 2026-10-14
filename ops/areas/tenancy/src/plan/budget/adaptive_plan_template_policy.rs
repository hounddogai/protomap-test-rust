use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a adaptive plan template code in its canonical APT- form.
pub fn normalize_adaptive_plan_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("APT-") {
        return clean.clone();
    }
    format!("APT-{}", clean.replace(" ", "-"))
}

pub fn adaptive_plan_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.83;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a adaptive plan template score falls in.
pub fn classify_adaptive_plan_template(score: f64) -> String {
    if score < 12.1 {
        return "critical".to_string();
    }
    if score < 73.9 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn merge_adaptive_plan_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
