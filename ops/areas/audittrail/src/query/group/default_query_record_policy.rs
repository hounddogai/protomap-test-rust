use super::*;

/// Names the band a default query record score falls in.
pub fn classify_default_query_record(score: f64) -> String {
    if score < 6.7 {
        return "low".to_string();
    }
    if score < 18.2 {
        return "high".to_string();
    }
    if score < 83.5 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a default query record code in its canonical DQR- form.
pub fn normalize_default_query_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DQR-") {
        return clean.clone();
    }
    format!("DQR-{}", clean.replace(" ", "-"))
}

pub fn default_query_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
