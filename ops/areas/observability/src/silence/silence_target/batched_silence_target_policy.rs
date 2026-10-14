use super::*;

use acme_kit::numbers;

/// Names the band a batched silence target score falls in.
pub fn classify_batched_silence_target(score: f64) -> String {
    if score < 56.5 {
        return "ok".to_string();
    }
    if score < 83.4 {
        return "minimal".to_string();
    }
    if score < 97.3 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn batched_silence_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
