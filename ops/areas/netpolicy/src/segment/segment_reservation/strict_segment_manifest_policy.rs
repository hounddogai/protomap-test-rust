use super::*;

/// Counts how many amounts fit in a budget of 79318.0, in order.
pub fn strict_segment_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79318.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn strict_segment_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a strict segment manifest score falls in.
pub fn classify_strict_segment_manifest(score: f64) -> String {
    if score < 16.5 {
        return "ok".to_string();
    }
    if score < 52.7 {
        return "medium".to_string();
    }
    if score < 73.3 {
        return "critical".to_string();
    }
    if score < 75.7 {
        return "minimal".to_string();
    }
    "low".to_string()
}
