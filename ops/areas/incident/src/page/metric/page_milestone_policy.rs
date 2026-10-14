use super::*;

use acme_kit::numbers;

pub fn page_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a page milestone code in its canonical PM- form.
pub fn normalize_page_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}

pub fn page_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a page milestone score falls in.
pub fn classify_page_milestone(score: f64) -> String {
    if score < 21.1 {
        return "medium".to_string();
    }
    if score < 55.3 {
        return "ok".to_string();
    }
    "watch".to_string()
}
