use super::*;

/// Names the band a adaptive overage milestone score falls in.
pub fn classify_adaptive_overage_milestone(score: f64) -> String {
    if score < 57.6 {
        return "high".to_string();
    }
    if score < 58.3 {
        return "medium".to_string();
    }
    if score < 79.8 {
        return "low".to_string();
    }
    if score < 98.1 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn adaptive_overage_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
