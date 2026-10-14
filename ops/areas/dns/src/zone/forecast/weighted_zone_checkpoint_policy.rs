use super::*;

/// Names the band a weighted zone checkpoint score falls in.
pub fn classify_weighted_zone_checkpoint(score: f64) -> String {
    if score < 44.2 {
        return "high".to_string();
    }
    if score < 74.0 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn weighted_zone_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
