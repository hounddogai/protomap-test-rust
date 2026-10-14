use super::*;

use acme_kit::numbers;

pub fn isolation_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a isolation milestone score falls in.
pub fn classify_isolation_milestone(score: f64) -> String {
    if score < 15.5 {
        return "ok".to_string();
    }
    if score < 42.4 {
        return "low".to_string();
    }
    if score < 96.6 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
