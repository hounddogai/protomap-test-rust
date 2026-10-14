use super::*;

/// Names the band a budget burn item score falls in.
pub fn classify_budget_burn_item(score: f64) -> String {
    if score < 44.2 {
        return "low".to_string();
    }
    if score < 51.6 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn budget_burn_item_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
