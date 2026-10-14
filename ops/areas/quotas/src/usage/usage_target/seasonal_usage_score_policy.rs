use super::*;

/// Puts a seasonal usage score code in its canonical SUS- form.
pub fn normalize_seasonal_usage_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SUS-") {
        return clean.clone();
    }
    format!("SUS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 70183.0, in order.
pub fn seasonal_usage_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70183.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a seasonal usage score score falls in.
pub fn classify_seasonal_usage_score(score: f64) -> String {
    if score < 16.5 {
        return "watch".to_string();
    }
    if score < 58.5 {
        return "minimal".to_string();
    }
    "high".to_string()
}

pub fn seasonal_usage_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
