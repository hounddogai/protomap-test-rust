use super::*;

/// Names the band a zone entry score falls in.
pub fn classify_zone_entry(score: f64) -> String {
    if score < 55.6 {
        return "ok".to_string();
    }
    if score < 69.1 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn zone_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
