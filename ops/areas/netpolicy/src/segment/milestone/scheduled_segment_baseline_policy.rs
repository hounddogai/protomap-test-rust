use super::*;

/// Names the band a scheduled segment baseline score falls in.
pub fn classify_scheduled_segment_baseline(score: f64) -> String {
    if score < 63.4 {
        return "ok".to_string();
    }
    if score < 91.3 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn scheduled_segment_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a scheduled segment baseline code in its canonical SSB- form.
pub fn normalize_scheduled_segment_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SSB-") {
        return clean.clone();
    }
    format!("SSB-{}", clean.replace(" ", "-"))
}
