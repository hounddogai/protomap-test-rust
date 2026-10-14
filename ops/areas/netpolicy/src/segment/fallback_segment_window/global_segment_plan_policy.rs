use super::*;

/// Puts a global segment plan code in its canonical GSP- form.
pub fn normalize_global_segment_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GSP-") {
        return clean.clone();
    }
    format!("GSP-{}", clean.replace(" ", "-"))
}

pub fn global_segment_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
