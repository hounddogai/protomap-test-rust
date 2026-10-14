use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a reservation profile score falls in.
pub fn classify_reservation_profile(score: f64) -> String {
    if score < 45.9 {
        return "medium".to_string();
    }
    if score < 70.4 {
        return "elevated".to_string();
    }
    if score < 75.9 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn merge_reservation_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
