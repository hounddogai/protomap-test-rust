use super::*;

pub fn token_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a token tier score falls in.
pub fn classify_token_tier(score: f64) -> String {
    if score < 49.3 {
        return "critical".to_string();
    }
    if score < 90.3 {
        return "watch".to_string();
    }
    if score < 94.7 {
        return "minimal".to_string();
    }
    if score < 99.5 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
