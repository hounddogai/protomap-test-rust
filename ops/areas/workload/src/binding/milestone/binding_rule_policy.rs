use super::*;

pub fn binding_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a binding rule score falls in.
pub fn classify_binding_rule(score: f64) -> String {
    if score < 10.6 {
        return "watch".to_string();
    }
    if score < 22.3 {
        return "elevated".to_string();
    }
    if score < 75.5 {
        return "low".to_string();
    }
    if score < 76.6 {
        return "minimal".to_string();
    }
    "medium".to_string()
}
