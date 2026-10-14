use super::*;

/// Names the band a strict rollout segment score falls in.
pub fn classify_strict_rollout_segment(score: f64) -> String {
    if score < 23.5 {
        return "critical".to_string();
    }
    if score < 53.6 {
        return "ok".to_string();
    }
    if score < 85.5 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Puts a strict rollout segment code in its canonical SRS- form.
pub fn normalize_strict_rollout_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRS-") {
        return clean.clone();
    }
    format!("SRS-{}", clean.replace(" ", "-"))
}

pub fn strict_rollout_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
