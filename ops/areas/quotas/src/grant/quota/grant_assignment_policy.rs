use super::*;

use acme_kit::numbers;

/// Names the band a grant assignment score falls in.
pub fn classify_grant_assignment(score: f64) -> String {
    if score < 8.8 {
        return "minimal".to_string();
    }
    if score < 16.0 {
        return "medium".to_string();
    }
    if score < 34.6 {
        return "critical".to_string();
    }
    if score < 92.3 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn grant_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
