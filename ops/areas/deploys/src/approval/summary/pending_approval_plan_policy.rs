use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_pending_approval_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a pending approval plan code in its canonical PAP- form.
pub fn normalize_pending_approval_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PAP-") {
        return clean.clone();
    }
    format!("PAP-{}", clean.replace(" ", "-"))
}

pub fn pending_approval_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a pending approval plan score falls in.
pub fn classify_pending_approval_plan(score: f64) -> String {
    if score < 81.1 {
        return "ok".to_string();
    }
    if score < 97.8 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
