use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_freeze_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a freeze schedule code in its canonical FS- form.
pub fn normalize_freeze_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FS-") {
        return clean.clone();
    }
    format!("FS-{}", clean.replace(" ", "-"))
}

/// Names the band a freeze schedule score falls in.
pub fn classify_freeze_schedule(score: f64) -> String {
    if score < 11.3 {
        return "low".to_string();
    }
    if score < 75.2 {
        return "medium".to_string();
    }
    "ok".to_string()
}
