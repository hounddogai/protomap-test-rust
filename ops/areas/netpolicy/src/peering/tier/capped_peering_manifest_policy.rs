use super::*;

use acme_kit::numbers;

/// Names the band a capped peering manifest score falls in.
pub fn classify_capped_peering_manifest(score: f64) -> String {
    if score < 53.6 {
        return "elevated".to_string();
    }
    if score < 91.6 {
        return "low".to_string();
    }
    if score < 93.0 {
        return "medium".to_string();
    }
    if score < 98.8 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn capped_peering_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn capped_peering_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
