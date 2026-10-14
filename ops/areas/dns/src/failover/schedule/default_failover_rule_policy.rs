use super::*;

/// Puts a default failover rule code in its canonical DFR- form.
pub fn normalize_default_failover_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DFR-") {
        return clean.clone();
    }
    format!("DFR-{}", clean.replace(" ", "-"))
}

pub fn default_failover_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
