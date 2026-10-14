use super::*;

/// Names the band a batched page baseline score falls in.
pub fn classify_batched_page_baseline(score: f64) -> String {
    if score < 38.6 {
        return "ok".to_string();
    }
    if score < 54.9 {
        return "medium".to_string();
    }
    if score < 79.2 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a batched page baseline code in its canonical BPB- form.
pub fn normalize_batched_page_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BPB-") {
        return clean.clone();
    }
    format!("BPB-{}", clean.replace(" ", "-"))
}

pub fn batched_page_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
