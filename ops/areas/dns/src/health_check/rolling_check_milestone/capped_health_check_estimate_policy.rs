use super::*;

/// Puts a capped health check estimate code in its canonical CHCE- form.
pub fn normalize_capped_health_check_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CHCE-") {
        return clean.clone();
    }
    format!("CHCE-{}", clean.replace(" ", "-"))
}

pub fn capped_health_check_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
