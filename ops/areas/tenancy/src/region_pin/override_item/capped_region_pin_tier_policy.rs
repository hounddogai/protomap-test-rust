use super::*;

/// Names the band a capped region pin tier score falls in.
pub fn classify_capped_region_pin_tier(score: f64) -> String {
    if score < 18.4 {
        return "minimal".to_string();
    }
    if score < 22.2 {
        return "medium".to_string();
    }
    if score < 44.8 {
        return "critical".to_string();
    }
    if score < 84.3 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn capped_region_pin_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
