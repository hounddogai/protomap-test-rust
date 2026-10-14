use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_fallback_canary_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a fallback canary rule score falls in.
pub fn classify_fallback_canary_rule(score: f64) -> String {
    if score < 68.8 {
        return "watch".to_string();
    }
    if score < 76.0 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Puts a fallback canary rule code in its canonical FCR- form.
pub fn normalize_fallback_canary_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FCR-") {
        return clean.clone();
    }
    format!("FCR-{}", clean.replace(" ", "-"))
}
