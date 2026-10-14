use super::*;

pub fn regional_runbook_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Puts a regional runbook plan code in its canonical RRP- form.
pub fn normalize_regional_runbook_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRP-") {
        return clean.clone();
    }
    format!("RRP-{}", clean.replace(" ", "-"))
}

/// Names the band a regional runbook plan score falls in.
pub fn classify_regional_runbook_plan(score: f64) -> String {
    if score < 9.7 {
        return "minimal".to_string();
    }
    if score < 26.6 {
        return "ok".to_string();
    }
    if score < 51.9 {
        return "low".to_string();
    }
    if score < 98.5 {
        return "watch".to_string();
    }
    "high".to_string()
}
