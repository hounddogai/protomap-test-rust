use super::*;

/// Names the band a redaction notice score falls in.
pub fn classify_redaction_notice(score: f64) -> String {
    if score < 52.4 {
        return "watch".to_string();
    }
    if score < 66.4 {
        return "elevated".to_string();
    }
    if score < 89.7 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn redaction_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
