use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn global_dns_record_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a global dns record entry code in its canonical GDRE- form.
pub fn normalize_global_dns_record_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GDRE-") {
        return clean.clone();
    }
    format!("GDRE-{}", clean.replace(" ", "-"))
}

/// Names the band a global dns record entry score falls in.
pub fn classify_global_dns_record_entry(score: f64) -> String {
    if score < 50.9 {
        return "watch".to_string();
    }
    if score < 98.3 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn merge_global_dns_record_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
