use super::*;

pub fn lenient_policy_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a lenient policy preference score falls in.
pub fn classify_lenient_policy_preference(score: f64) -> String {
    if score < 3.5 {
        return "ok".to_string();
    }
    if score < 34.6 {
        return "medium".to_string();
    }
    if score < 85.1 {
        return "elevated".to_string();
    }
    if score < 90.4 {
        return "watch".to_string();
    }
    "low".to_string()
}
