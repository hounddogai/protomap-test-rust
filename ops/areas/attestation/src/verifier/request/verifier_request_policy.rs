use super::*;

use acme_kit::numbers;

/// Names the band a verifier request score falls in.
pub fn classify_verifier_request(score: f64) -> String {
    if score < 31.6 {
        return "elevated".to_string();
    }
    if score < 44.5 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn verifier_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
