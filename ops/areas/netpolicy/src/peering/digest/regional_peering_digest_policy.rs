use super::*;

/// Names the band a regional peering digest score falls in.
pub fn classify_regional_peering_digest(score: f64) -> String {
    if score < 15.4 {
        return "minimal".to_string();
    }
    if score < 43.6 {
        return "critical".to_string();
    }
    if score < 73.9 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn regional_peering_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
