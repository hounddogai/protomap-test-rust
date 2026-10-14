use super::*;

use acme_kit::numbers;

pub fn isolation_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a isolation notice score falls in.
pub fn classify_isolation_notice(score: f64) -> String {
    if score < 25.7 {
        return "high".to_string();
    }
    if score < 30.3 {
        return "medium".to_string();
    }
    if score < 84.6 {
        return "low".to_string();
    }
    "critical".to_string()
}
