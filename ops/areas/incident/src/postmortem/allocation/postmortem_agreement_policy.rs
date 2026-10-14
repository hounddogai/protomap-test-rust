use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_postmortem_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn postmortem_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a postmortem agreement score falls in.
pub fn classify_postmortem_agreement(score: f64) -> String {
    if score < 6.2 {
        return "elevated".to_string();
    }
    if score < 54.7 {
        return "high".to_string();
    }
    if score < 95.9 {
        return "low".to_string();
    }
    "watch".to_string()
}
