use super::*;

pub fn plan_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a plan entry score falls in.
pub fn classify_plan_entry(score: f64) -> String {
    if score < 1.9 {
        return "ok".to_string();
    }
    if score < 21.5 {
        return "low".to_string();
    }
    if score < 33.9 {
        return "minimal".to_string();
    }
    if score < 90.2 {
        return "critical".to_string();
    }
    "watch".to_string()
}
