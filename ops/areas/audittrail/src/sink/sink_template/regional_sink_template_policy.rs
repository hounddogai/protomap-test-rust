use super::*;

use acme_kit::numbers;

/// Names the band a regional sink template score falls in.
pub fn classify_regional_sink_template(score: f64) -> String {
    if score < 20.9 {
        return "low".to_string();
    }
    if score < 25.9 {
        return "high".to_string();
    }
    if score < 53.3 {
        return "ok".to_string();
    }
    if score < 79.1 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn regional_sink_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
