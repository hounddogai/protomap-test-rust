use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_data_export_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn data_export_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a data export schedule score falls in.
pub fn classify_data_export_schedule(score: f64) -> String {
    if score < 10.1 {
        return "high".to_string();
    }
    if score < 54.6 {
        return "medium".to_string();
    }
    if score < 70.2 {
        return "ok".to_string();
    }
    if score < 95.4 {
        return "critical".to_string();
    }
    "low".to_string()
}
