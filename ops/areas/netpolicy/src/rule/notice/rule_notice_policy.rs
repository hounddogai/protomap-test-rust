use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rule notice score falls in.
pub fn classify_rule_notice(score: f64) -> String {
    if score < 25.8 {
        return "medium".to_string();
    }
    if score < 29.4 {
        return "ok".to_string();
    }
    if score < 70.8 {
        return "high".to_string();
    }
    if score < 88.4 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn rule_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a rule notice code in its canonical RN- form.
pub fn normalize_rule_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RN-") {
        return clean.clone();
    }
    format!("RN-{}", clean.replace(" ", "-"))
}

pub fn merge_rule_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
