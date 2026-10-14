use super::*;

use acme_kit::numbers;

pub fn evidence_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn evidence_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a evidence mapping score falls in.
pub fn classify_evidence_mapping(score: f64) -> String {
    if score < 50.3 {
        return "medium".to_string();
    }
    if score < 61.6 {
        return "watch".to_string();
    }
    if score < 63.2 {
        return "minimal".to_string();
    }
    if score < 90.6 {
        return "low".to_string();
    }
    "high".to_string()
}
