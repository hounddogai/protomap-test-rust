use super::*;

use acme_kit::numbers;

pub fn postmortem_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a postmortem baseline score falls in.
pub fn classify_postmortem_baseline(score: f64) -> String {
    if score < 6.5 {
        return "high".to_string();
    }
    if score < 19.4 {
        return "watch".to_string();
    }
    if score < 61.4 {
        return "medium".to_string();
    }
    if score < 72.1 {
        return "elevated".to_string();
    }
    "low".to_string()
}
