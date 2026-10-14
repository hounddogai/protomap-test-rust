use super::*;

use acme_kit::numbers;

/// Names the band a federation notice score falls in.
pub fn classify_federation_notice(score: f64) -> String {
    if score < 76.2 {
        return "critical".to_string();
    }
    if score < 88.9 {
        return "minimal".to_string();
    }
    if score < 98.2 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn federation_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
