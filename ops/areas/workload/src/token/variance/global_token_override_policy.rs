use super::*;

use acme_kit::numbers;

pub fn global_token_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a global token override score falls in.
pub fn classify_global_token_override(score: f64) -> String {
    if score < 27.5 {
        return "medium".to_string();
    }
    if score < 91.0 {
        return "low".to_string();
    }
    if score < 93.5 {
        return "high".to_string();
    }
    "watch".to_string()
}
