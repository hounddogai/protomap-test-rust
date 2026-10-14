use super::*;

/// Names the band a weighted region pin profile score falls in.
pub fn classify_weighted_region_pin_profile(score: f64) -> String {
    if score < 14.3 {
        return "high".to_string();
    }
    if score < 47.7 {
        return "elevated".to_string();
    }
    if score < 54.2 {
        return "medium".to_string();
    }
    if score < 94.5 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn weighted_region_pin_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 90793.0, in order.
pub fn weighted_region_pin_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90793.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
