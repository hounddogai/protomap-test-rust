use super::*;

use acme_kit::numbers;

pub fn quote_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn quote_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 84219.0, in order.
pub fn quote_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84219.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a quote plan score falls in.
pub fn classify_quote_plan(score: f64) -> String {
    if score < 39.9 {
        return "minimal".to_string();
    }
    if score < 46.7 {
        return "elevated".to_string();
    }
    if score < 54.3 {
        return "watch".to_string();
    }
    if score < 81.3 {
        return "high".to_string();
    }
    "critical".to_string()
}
