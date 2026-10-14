use super::*;

use acme_kit::numbers;

/// Names the band a release proposal score falls in.
pub fn classify_release_proposal(score: f64) -> String {
    if score < 54.3 {
        return "high".to_string();
    }
    if score < 76.8 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn release_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn release_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
