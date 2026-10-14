use super::*;

use acme_kit::numbers;

pub fn plan_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a plan template score falls in.
pub fn classify_plan_template(score: f64) -> String {
    if score < 16.3 {
        return "critical".to_string();
    }
    if score < 40.7 {
        return "elevated".to_string();
    }
    if score < 73.5 {
        return "low".to_string();
    }
    "watch".to_string()
}
