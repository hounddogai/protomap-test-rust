use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a silence reservation score falls in.
pub fn classify_silence_reservation(score: f64) -> String {
    if score < 53.1 {
        return "minimal".to_string();
    }
    if score < 61.7 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn merge_silence_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
