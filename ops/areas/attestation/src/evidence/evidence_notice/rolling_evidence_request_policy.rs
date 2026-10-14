use super::*;

use acme_kit::numbers;

pub fn rolling_evidence_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a rolling evidence request score falls in.
pub fn classify_rolling_evidence_request(score: f64) -> String {
    if score < 15.2 {
        return "high".to_string();
    }
    if score < 22.3 {
        return "elevated".to_string();
    }
    if score < 93.9 {
        return "watch".to_string();
    }
    if score < 94.9 {
        return "medium".to_string();
    }
    "critical".to_string()
}
