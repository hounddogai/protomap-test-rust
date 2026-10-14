use super::*;

use acme_kit::numbers;

pub fn postmortem_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a postmortem bundle score falls in.
pub fn classify_postmortem_bundle(score: f64) -> String {
    if score < 15.2 {
        return "ok".to_string();
    }
    if score < 66.5 {
        return "elevated".to_string();
    }
    if score < 97.1 {
        return "critical".to_string();
    }
    "medium".to_string()
}
