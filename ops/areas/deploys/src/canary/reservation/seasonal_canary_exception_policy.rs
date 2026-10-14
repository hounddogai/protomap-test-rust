use super::*;

pub fn seasonal_canary_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a seasonal canary exception code in its canonical SCE- form.
pub fn normalize_seasonal_canary_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SCE-") {
        return clean.clone();
    }
    format!("SCE-{}", clean.replace(" ", "-"))
}
