use super::*;

pub fn adaptive_federation_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a adaptive federation cycle code in its canonical AFC- form.
pub fn normalize_adaptive_federation_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AFC-") {
        return clean.clone();
    }
    format!("AFC-{}", clean.replace(" ", "-"))
}
