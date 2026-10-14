use super::*;

/// Names the band a primary nonce digest score falls in.
pub fn classify_primary_nonce_digest(score: f64) -> String {
    if score < 27.6 {
        return "minimal".to_string();
    }
    if score < 28.5 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn primary_nonce_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
