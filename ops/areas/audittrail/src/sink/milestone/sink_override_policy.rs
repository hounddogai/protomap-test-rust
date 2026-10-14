use super::*;

use acme_kit::numbers;

/// Names the band a sink override score falls in.
pub fn classify_sink_override(score: f64) -> String {
    if score < 25.2 {
        return "ok".to_string();
    }
    if score < 35.5 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn sink_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
