use super::*;

use acme_kit::numbers;

pub fn primary_token_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a primary token review score falls in.
pub fn classify_primary_token_review(score: f64) -> String {
    if score < 7.7 {
        return "critical".to_string();
    }
    if score < 37.1 {
        return "high".to_string();
    }
    if score < 39.5 {
        return "minimal".to_string();
    }
    if score < 51.1 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
