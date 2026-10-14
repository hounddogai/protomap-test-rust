use super::*;

use acme_kit::numbers;

/// Names the band a manual flow log baseline score falls in.
pub fn classify_manual_flow_log_baseline(score: f64) -> String {
    if score < 24.1 {
        return "watch".to_string();
    }
    if score < 26.5 {
        return "minimal".to_string();
    }
    if score < 62.7 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn manual_flow_log_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
