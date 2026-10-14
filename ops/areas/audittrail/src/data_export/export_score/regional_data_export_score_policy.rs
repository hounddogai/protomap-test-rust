use super::*;

use acme_kit::numbers;

pub fn regional_data_export_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a regional data export score score falls in.
pub fn classify_regional_data_export_score(score: f64) -> String {
    if score < 27.5 {
        return "elevated".to_string();
    }
    if score < 77.3 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn regional_data_export_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
