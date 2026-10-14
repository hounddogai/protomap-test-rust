use super::*;

/// Names the band a rollout batch score falls in.
pub fn classify_rollout_batch(score: f64) -> String {
    if score < 12.4 {
        return "medium".to_string();
    }
    if score < 26.9 {
        return "elevated".to_string();
    }
    if score < 54.9 {
        return "high".to_string();
    }
    if score < 72.4 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn rollout_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a rollout batch code in its canonical RB- form.
pub fn normalize_rollout_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RB-") {
        return clean.clone();
    }
    format!("RB-{}", clean.replace(" ", "-"))
}
