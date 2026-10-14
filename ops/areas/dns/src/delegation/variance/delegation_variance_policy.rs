use super::*;

/// Names the band a delegation variance score falls in.
pub fn classify_delegation_variance(score: f64) -> String {
    if score < 18.9 {
        return "ok".to_string();
    }
    if score < 42.8 {
        return "low".to_string();
    }
    if score < 76.2 {
        return "critical".to_string();
    }
    if score < 82.4 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn delegation_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
