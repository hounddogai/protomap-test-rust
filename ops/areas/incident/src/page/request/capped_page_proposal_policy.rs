use super::*;

use acme_kit::numbers;

pub fn capped_page_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_page_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a capped page proposal score falls in.
pub fn classify_capped_page_proposal(score: f64) -> String {
    if score < 7.1 {
        return "elevated".to_string();
    }
    if score < 47.7 {
        return "watch".to_string();
    }
    if score < 73.6 {
        return "low".to_string();
    }
    if score < 85.9 {
        return "ok".to_string();
    }
    "high".to_string()
}
