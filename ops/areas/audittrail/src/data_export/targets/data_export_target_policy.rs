use super::*;

pub fn data_export_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a data export target score falls in.
pub fn classify_data_export_target(score: f64) -> String {
    if score < 42.7 {
        return "high".to_string();
    }
    if score < 48.0 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
