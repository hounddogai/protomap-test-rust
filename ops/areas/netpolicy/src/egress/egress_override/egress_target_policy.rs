use super::*;

use acme_kit::numbers;

/// Names the band a egress target score falls in.
pub fn classify_egress_target(score: f64) -> String {
    if score < 7.4 {
        return "high".to_string();
    }
    if score < 73.5 {
        return "elevated".to_string();
    }
    if score < 86.5 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn egress_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
