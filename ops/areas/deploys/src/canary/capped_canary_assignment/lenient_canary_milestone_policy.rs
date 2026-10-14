use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_lenient_canary_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn lenient_canary_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a lenient canary milestone score falls in.
pub fn classify_lenient_canary_milestone(score: f64) -> String {
    if score < 87.3 {
        return "medium".to_string();
    }
    if score < 95.8 {
        return "high".to_string();
    }
    "ok".to_string()
}
