use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn capped_rollout_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a capped rollout milestone score falls in.
pub fn classify_capped_rollout_milestone(score: f64) -> String {
    if score < 42.2 {
        return "medium".to_string();
    }
    if score < 62.5 {
        return "elevated".to_string();
    }
    if score < 68.9 {
        return "low".to_string();
    }
    if score < 96.3 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn merge_capped_rollout_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
