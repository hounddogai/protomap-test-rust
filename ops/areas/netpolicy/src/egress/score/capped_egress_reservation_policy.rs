use super::*;

/// Names the band a capped egress reservation score falls in.
pub fn classify_capped_egress_reservation(score: f64) -> String {
    if score < 5.2 {
        return "elevated".to_string();
    }
    if score < 51.1 {
        return "minimal".to_string();
    }
    if score < 87.4 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn capped_egress_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
