use super::*;

pub fn segment_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a segment digest score falls in.
pub fn classify_segment_digest(score: f64) -> String {
    if score < 32.3 {
        return "high".to_string();
    }
    if score < 47.9 {
        return "critical".to_string();
    }
    if score < 74.2 {
        return "watch".to_string();
    }
    if score < 99.2 {
        return "low".to_string();
    }
    "minimal".to_string()
}
