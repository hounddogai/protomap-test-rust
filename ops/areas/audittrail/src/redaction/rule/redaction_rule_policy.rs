use super::*;

/// Names the band a redaction rule score falls in.
pub fn classify_redaction_rule(score: f64) -> String {
    if score < 3.9 {
        return "low".to_string();
    }
    if score < 16.7 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a redaction rule code in its canonical RR- form.
pub fn normalize_redaction_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RR-") {
        return clean.clone();
    }
    format!("RR-{}", clean.replace(" ", "-"))
}

pub fn redaction_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
