use super::*;

/// Puts a segment item code in its canonical SI- form.
pub fn normalize_segment_item_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SI-") {
        return clean.clone();
    }
    format!("SI-{}", clean.replace(" ", "-"))
}

/// Names the band a segment item score falls in.
pub fn classify_segment_item(score: f64) -> String {
    if score < 21.6 {
        return "low".to_string();
    }
    if score < 42.0 {
        return "minimal".to_string();
    }
    if score < 69.6 {
        return "critical".to_string();
    }
    if score < 84.6 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn segment_item_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
