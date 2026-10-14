use super::*;

pub fn tiered_silence_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Names the band a tiered silence target score falls in.
pub fn classify_tiered_silence_target(score: f64) -> String {
    if score < 15.7 {
        return "medium".to_string();
    }
    if score < 47.9 {
        return "elevated".to_string();
    }
    "low".to_string()
}
