use super::*;

pub fn fallback_egress_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a fallback egress override score falls in.
pub fn classify_fallback_egress_override(score: f64) -> String {
    if score < 13.7 {
        return "watch".to_string();
    }
    if score < 20.0 {
        return "high".to_string();
    }
    if score < 78.1 {
        return "minimal".to_string();
    }
    "low".to_string()
}
