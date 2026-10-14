use super::*;

use acme_kit::numbers;

pub fn capped_page_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_page_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Names the band a capped page budget score falls in.
pub fn classify_capped_page_budget(score: f64) -> String {
    if score < 38.9 {
        return "medium".to_string();
    }
    if score < 43.1 {
        return "ok".to_string();
    }
    if score < 49.8 {
        return "minimal".to_string();
    }
    if score < 97.7 {
        return "critical".to_string();
    }
    "watch".to_string()
}
