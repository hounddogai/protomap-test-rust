use super::*;

/// Counts how many amounts fit in a budget of 32734.0, in order.
pub fn sink_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32734.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a sink segment score falls in.
pub fn classify_sink_segment(score: f64) -> String {
    if score < 13.3 {
        return "ok".to_string();
    }
    if score < 33.1 {
        return "critical".to_string();
    }
    if score < 71.3 {
        return "watch".to_string();
    }
    if score < 76.4 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn sink_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
