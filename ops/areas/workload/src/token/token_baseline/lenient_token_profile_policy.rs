use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a lenient token profile score falls in.
pub fn classify_lenient_token_profile(score: f64) -> String {
    if score < 13.7 {
        return "elevated".to_string();
    }
    if score < 70.8 {
        return "minimal".to_string();
    }
    if score < 71.9 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_lenient_token_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
