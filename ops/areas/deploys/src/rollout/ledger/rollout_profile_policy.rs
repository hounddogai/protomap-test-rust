use super::*;

/// Names the band a rollout profile score falls in.
pub fn classify_rollout_profile(score: f64) -> String {
    if score < 42.1 {
        return "low".to_string();
    }
    if score < 50.2 {
        return "ok".to_string();
    }
    if score < 58.6 {
        return "medium".to_string();
    }
    if score < 82.8 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn rollout_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
