use super::*;

use acme_kit::numbers;

/// Names the band a capped retention notice score falls in.
pub fn classify_capped_retention_notice(score: f64) -> String {
    if score < 21.1 {
        return "critical".to_string();
    }
    if score < 22.7 {
        return "minimal".to_string();
    }
    if score < 65.7 {
        return "ok".to_string();
    }
    if score < 92.8 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn capped_retention_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
