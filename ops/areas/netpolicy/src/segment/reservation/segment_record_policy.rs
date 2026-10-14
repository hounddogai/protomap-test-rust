use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn segment_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn merge_segment_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn segment_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
