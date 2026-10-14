use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn lenient_flow_log_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_lenient_flow_log_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a lenient flow log proposal score falls in.
pub fn classify_lenient_flow_log_proposal(score: f64) -> String {
    if score < 40.0 {
        return "medium".to_string();
    }
    if score < 52.4 {
        return "low".to_string();
    }
    "ok".to_string()
}
