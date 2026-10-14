use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_flow_log_window_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a flow log window score falls in.
pub fn classify_flow_log_window(score: f64) -> String {
    if score < 12.4 {
        return "critical".to_string();
    }
    if score < 18.0 {
        return "medium".to_string();
    }
    if score < 29.6 {
        return "minimal".to_string();
    }
    if score < 84.8 {
        return "high".to_string();
    }
    "low".to_string()
}
