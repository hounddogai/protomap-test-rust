use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn trust_domain_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_trust_domain_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a trust domain exception code in its canonical TDE- form.
pub fn normalize_trust_domain_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDE-") {
        return clean.clone();
    }
    format!("TDE-{}", clean.replace(" ", "-"))
}
