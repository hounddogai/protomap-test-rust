use super::*;

use acme_kit::numbers;

/// Names the band a rollback checkpoint score falls in.
pub fn classify_rollback_checkpoint(score: f64) -> String {
    if score < 2.3 {
        return "high".to_string();
    }
    if score < 88.1 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn rollback_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
