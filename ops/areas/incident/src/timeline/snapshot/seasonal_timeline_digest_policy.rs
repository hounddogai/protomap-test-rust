use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_seasonal_timeline_digest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal timeline digest score falls in.
pub fn classify_seasonal_timeline_digest(score: f64) -> String {
    if score < 21.6 {
        return "ok".to_string();
    }
    if score < 51.3 {
        return "minimal".to_string();
    }
    "medium".to_string()
}
