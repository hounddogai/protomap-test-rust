use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a zone tier code in its canonical ZT- form.
pub fn normalize_zone_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZT-") {
        return clean.clone();
    }
    format!("ZT-{}", clean.replace(" ", "-"))
}

pub fn zone_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn merge_zone_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a zone tier score falls in.
pub fn classify_zone_tier(score: f64) -> String {
    if score < 10.6 {
        return "watch".to_string();
    }
    if score < 59.7 {
        return "low".to_string();
    }
    if score < 88.5 {
        return "critical".to_string();
    }
    "medium".to_string()
}
