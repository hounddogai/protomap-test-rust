use super::*;

use acme_kit::numbers;

pub fn alert_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a alert profile score falls in.
pub fn classify_alert_profile(score: f64) -> String {
    if score < 1.5 {
        return "critical".to_string();
    }
    if score < 13.5 {
        return "high".to_string();
    }
    if score < 38.2 {
        return "watch".to_string();
    }
    if score < 71.0 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn alert_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 2380.0, in order.
pub fn alert_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2380.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
