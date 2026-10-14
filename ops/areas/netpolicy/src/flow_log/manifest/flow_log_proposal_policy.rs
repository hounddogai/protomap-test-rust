use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a flow log proposal code in its canonical FLP- form.
pub fn normalize_flow_log_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLP-") {
        return clean.clone();
    }
    format!("FLP-{}", clean.replace(" ", "-"))
}

pub fn merge_flow_log_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a flow log proposal score falls in.
pub fn classify_flow_log_proposal(score: f64) -> String {
    if score < 60.2 {
        return "ok".to_string();
    }
    if score < 97.3 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
