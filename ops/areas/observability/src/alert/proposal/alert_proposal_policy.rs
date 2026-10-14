use super::*;

use acme_kit::numbers;

pub fn alert_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a alert proposal score falls in.
pub fn classify_alert_proposal(score: f64) -> String {
    if score < 34.4 {
        return "low".to_string();
    }
    if score < 75.7 {
        return "high".to_string();
    }
    "watch".to_string()
}
