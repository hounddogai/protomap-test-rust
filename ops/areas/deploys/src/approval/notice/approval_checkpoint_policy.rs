use super::*;

/// Puts a approval checkpoint code in its canonical AC- form.
pub fn normalize_approval_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AC-") {
        return clean.clone();
    }
    format!("AC-{}", clean.replace(" ", "-"))
}

/// Names the band a approval checkpoint score falls in.
pub fn classify_approval_checkpoint(score: f64) -> String {
    if score < 24.8 {
        return "watch".to_string();
    }
    if score < 81.2 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn approval_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
