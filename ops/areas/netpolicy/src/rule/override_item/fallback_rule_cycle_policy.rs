use super::*;

pub fn fallback_rule_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a fallback rule cycle score falls in.
pub fn classify_fallback_rule_cycle(score: f64) -> String {
    if score < 4.9 {
        return "elevated".to_string();
    }
    if score < 49.0 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
