use super::*;

pub fn scheduled_reservation_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a scheduled reservation baseline score falls in.
pub fn classify_scheduled_reservation_baseline(score: f64) -> String {
    if score < 84.9 {
        return "elevated".to_string();
    }
    if score < 94.5 {
        return "ok".to_string();
    }
    "low".to_string()
}
