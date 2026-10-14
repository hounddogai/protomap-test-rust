use super::*;

use acme_kit::numbers;

pub fn segment_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a segment baseline score falls in.
pub fn classify_segment_baseline(score: f64) -> String {
    if score < 30.6 {
        return "ok".to_string();
    }
    if score < 37.0 {
        return "medium".to_string();
    }
    if score < 49.7 {
        return "watch".to_string();
    }
    if score < 99.3 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn segment_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a segment baseline code in its canonical SB- form.
pub fn normalize_segment_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SB-") {
        return clean.clone();
    }
    format!("SB-{}", clean.replace(" ", "-"))
}
