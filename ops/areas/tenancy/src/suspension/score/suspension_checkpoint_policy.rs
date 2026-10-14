use super::*;

pub fn suspension_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a suspension checkpoint code in its canonical SC- form.
pub fn normalize_suspension_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SC-") {
        return clean.clone();
    }
    format!("SC-{}", clean.replace(" ", "-"))
}

/// Names the band a suspension checkpoint score falls in.
pub fn classify_suspension_checkpoint(score: f64) -> String {
    if score < 6.2 {
        return "critical".to_string();
    }
    if score < 40.5 {
        return "low".to_string();
    }
    if score < 42.6 {
        return "watch".to_string();
    }
    if score < 76.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
