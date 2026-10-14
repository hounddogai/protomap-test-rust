use super::*;

use acme_kit::numbers;

/// Names the band a strict peering draft score falls in.
pub fn classify_strict_peering_draft(score: f64) -> String {
    if score < 5.5 {
        return "medium".to_string();
    }
    if score < 19.3 {
        return "high".to_string();
    }
    if score < 44.3 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn strict_peering_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn strict_peering_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
