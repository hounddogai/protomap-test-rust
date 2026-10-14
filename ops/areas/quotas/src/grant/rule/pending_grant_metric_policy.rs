use super::*;

use acme_kit::numbers;

/// Names the band a pending grant metric score falls in.
pub fn classify_pending_grant_metric(score: f64) -> String {
    if score < 14.8 {
        return "watch".to_string();
    }
    if score < 18.9 {
        return "minimal".to_string();
    }
    if score < 80.8 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn pending_grant_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
