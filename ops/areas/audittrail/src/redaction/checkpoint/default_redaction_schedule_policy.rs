use super::*;

/// Names the band a default redaction schedule score falls in.
pub fn classify_default_redaction_schedule(score: f64) -> String {
    if score < 26.8 {
        return "minimal".to_string();
    }
    if score < 81.1 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Puts a default redaction schedule code in its canonical DRS- form.
pub fn normalize_default_redaction_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRS-") {
        return clean.clone();
    }
    format!("DRS-{}", clean.replace(" ", "-"))
}

pub fn default_redaction_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
