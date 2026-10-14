use super::*;

/// Names the band a rollback proposal score falls in.
pub fn classify_rollback_proposal(score: f64) -> String {
    if score < 61.7 {
        return "elevated".to_string();
    }
    if score < 75.5 {
        return "medium".to_string();
    }
    if score < 81.5 {
        return "minimal".to_string();
    }
    if score < 96.9 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn rollback_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
