use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn tiered_flow_log_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a tiered flow log audit score falls in.
pub fn classify_tiered_flow_log_audit(score: f64) -> String {
    if score < 11.2 {
        return "ok".to_string();
    }
    if score < 33.1 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Puts a tiered flow log audit code in its canonical TFLA- form.
pub fn normalize_tiered_flow_log_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFLA-") {
        return clean.clone();
    }
    format!("TFLA-{}", clean.replace(" ", "-"))
}

pub fn merge_tiered_flow_log_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
