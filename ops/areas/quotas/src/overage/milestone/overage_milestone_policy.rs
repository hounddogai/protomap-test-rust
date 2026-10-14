use super::*;

pub fn overage_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a overage milestone code in its canonical OM- form.
pub fn normalize_overage_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OM-") {
        return clean.clone();
    }
    format!("OM-{}", clean.replace(" ", "-"))
}
