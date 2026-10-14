use super::*;

use acme_kit::numbers;

pub fn data_export_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn data_export_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a data export notice score falls in.
pub fn classify_data_export_notice(score: f64) -> String {
    if score < 20.9 {
        return "elevated".to_string();
    }
    if score < 64.0 {
        return "ok".to_string();
    }
    if score < 71.7 {
        return "minimal".to_string();
    }
    if score < 89.0 {
        return "watch".to_string();
    }
    "low".to_string()
}
