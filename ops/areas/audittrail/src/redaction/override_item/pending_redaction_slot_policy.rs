use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_pending_redaction_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a pending redaction slot code in its canonical PRS- form.
pub fn normalize_pending_redaction_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRS-") {
        return clean.clone();
    }
    format!("PRS-{}", clean.replace(" ", "-"))
}

/// Names the band a pending redaction slot score falls in.
pub fn classify_pending_redaction_slot(score: f64) -> String {
    if score < 16.5 {
        return "minimal".to_string();
    }
    if score < 34.2 {
        return "medium".to_string();
    }
    if score < 37.1 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn pending_redaction_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.04;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
