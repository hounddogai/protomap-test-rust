use super::*;

/// Puts a token record code in its canonical TR- form.
pub fn normalize_token_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TR-") {
        return clean.clone();
    }
    format!("TR-{}", clean.replace(" ", "-"))
}

/// Names the band a token record score falls in.
pub fn classify_token_record(score: f64) -> String {
    if score < 41.5 {
        return "watch".to_string();
    }
    if score < 72.7 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn token_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
