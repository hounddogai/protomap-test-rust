use super::*;

use acme_kit::numbers;

/// Names the band a silence slot score falls in.
pub fn classify_silence_slot(score: f64) -> String {
    if score < 42.3 {
        return "high".to_string();
    }
    if score < 89.1 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn silence_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
