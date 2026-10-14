use super::*;

use acme_kit::numbers;

pub fn primary_rule_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a primary rule notice score falls in.
pub fn classify_primary_rule_notice(score: f64) -> String {
    if score < 12.4 {
        return "medium".to_string();
    }
    if score < 20.5 {
        return "minimal".to_string();
    }
    if score < 65.2 {
        return "critical".to_string();
    }
    "ok".to_string()
}
