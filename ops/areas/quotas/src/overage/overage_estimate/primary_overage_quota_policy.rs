use super::*;

pub fn primary_overage_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a primary overage quota code in its canonical POQ- form.
pub fn normalize_primary_overage_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("POQ-") {
        return clean.clone();
    }
    format!("POQ-{}", clean.replace(" ", "-"))
}
